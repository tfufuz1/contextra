//! Erasure and deletion guarantee port traits and types (§5.3, §5.4, §5.5, §8.3, §8.4).
//!
//! Provides abstract contracts for multi-tier document erasure:
//! - [`ErasureMode`]: System-wide erasure watermark (`Off`, `Assured`, `Attested`).
//! - [`ErasureBackend`]: Storage-level physical scope derivation and purge execution.
//! - [`ErasureHooks`]: Lifecycle callbacks during coordinator execution.
//! - [`LayerScan`]: Verification scan contracts for storage/index layers.
//! - [`RestoreGuard`]: Anti-resurrection guard for checkpoint restoration.
//! - [`PhysicalScope`]: Opaque tenant-encoded physical keys and prefixes.

#![forbid(unsafe_code)]

use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use contextra_types::{CollectionId, ContextraError, DocId, TenantId, TxId};

/// Operational mode for document erasure and deletion guarantees (§5.1, §8.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default)]
pub enum ErasureMode {
    /// Off (Default): Standard tombstone delete. No ledger, no fence, no footprint scan.
    #[default]
    Off,
    /// Assured: Ledger append before tombstone, purge-intent with resume, fence, restore barrier.
    Assured,
    /// Attested: Cryptographic witnesses from genuine scans, v4 proof issuance, audit trail.
    Attested,
}

impl ErasureMode {
    /// Returns true if this mode requires active erasure ledger recording.
    #[must_use]
    pub fn requires_ledger(&self) -> bool {
        matches!(self, Self::Assured | Self::Attested)
    }

    /// Returns true if this mode requires cryptographic proof generation.
    #[must_use]
    pub fn requires_attestation(&self) -> bool {
        matches!(self, Self::Attested)
    }
}

/// Unique identifier for a purge operation (§5.5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct PurgeId(pub [u8; 16]);

impl PurgeId {
    /// Creates a new `PurgeId` wrapping raw 16-byte identifier.
    pub const fn new(bytes: [u8; 16]) -> Self {
        Self(bytes)
    }
}

/// Logical scope of a deletion request specified by the caller (§5.4).
///
/// Every variant carries the owning [`TenantId`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogicalScope {
    /// Target document within a collection and tenant.
    Document {
        /// Tenant owning the document.
        tenant_id: TenantId,
        /// Collection containing the document.
        collection_id: CollectionId,
        /// Document identifier.
        doc_id: DocId,
    },
    /// An entire collection within a tenant.
    Collection {
        /// Tenant owning the collection.
        tenant_id: TenantId,
        /// Collection identifier.
        collection_id: CollectionId,
    },
    /// An entire tenant.
    Tenant {
        /// Tenant identifier.
        tenant_id: TenantId,
    },
}

impl LogicalScope {
    /// Returns the [`TenantId`] associated with this scope.
    #[must_use]
    pub fn tenant_id(&self) -> TenantId {
        match self {
            Self::Document { tenant_id, .. }
            | Self::Collection { tenant_id, .. }
            | Self::Tenant { tenant_id } => *tenant_id,
        }
    }
}

/// Opaque physical, tenant-encoded keys and prefixes (§5.4).
///
/// Fields are strictly private. The constructor is gated behind feature `backend-impl`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PhysicalScope {
    keys: Vec<Vec<u8>>,
    prefixes: Vec<Vec<u8>>,
}

impl PhysicalScope {
    /// Creates a new validated, canonical `PhysicalScope` (§5.4).
    ///
    /// # Invariants
    /// 1. Must not be empty (at least one key or prefix required).
    /// 2. Every key and prefix MUST start with `t:<tenant_id>:`. Empty prefix is rejected.
    /// 3. Canonical: sorted, deduplicated, and subsumed keys/prefixes removed.
    #[cfg(feature = "backend-impl")]
    pub fn try_new(
        tenant_id: TenantId,
        keys: Vec<Vec<u8>>,
        prefixes: Vec<Vec<u8>>,
    ) -> std::result::Result<Self, ContextraError> {
        if keys.is_empty() && prefixes.is_empty() {
            return Err(ContextraError::InvalidInput(
                "PhysicalScope cannot be empty".to_string(),
            ));
        }

        let tenant_prefix_bytes = format!("t:{}:", tenant_id.0).into_bytes();

        for k in &keys {
            if !k.starts_with(&tenant_prefix_bytes) {
                return Err(ContextraError::InvalidInput(format!(
                    "Key {:?} does not start with expected tenant prefix 't:{}:'",
                    k, tenant_id.0
                )));
            }
        }

        for p in &prefixes {
            if p.is_empty() || !p.starts_with(&tenant_prefix_bytes) {
                return Err(ContextraError::InvalidInput(format!(
                    "Prefix {:?} does not start with expected tenant prefix 't:{}:'",
                    p, tenant_id.0
                )));
            }
        }

        let mut sorted_prefixes = prefixes;
        sorted_prefixes.sort();
        sorted_prefixes.dedup();

        let mut canonical_prefixes: Vec<Vec<u8>> = Vec::new();
        for p in sorted_prefixes {
            let is_subsumed = canonical_prefixes
                .iter()
                .any(|existing| p.starts_with(existing.as_slice()));
            if !is_subsumed {
                canonical_prefixes.push(p);
            }
        }

        let mut sorted_keys = keys;
        sorted_keys.sort();
        sorted_keys.dedup();

        let canonical_keys: Vec<Vec<u8>> = sorted_keys
            .into_iter()
            .filter(|k| {
                !canonical_prefixes
                    .iter()
                    .any(|p| k.starts_with(p.as_slice()))
            })
            .collect();

        Ok(Self {
            keys: canonical_keys,
            prefixes: canonical_prefixes,
        })
    }

    /// Returns the explicit physical keys in this scope.
    #[must_use]
    pub fn keys(&self) -> &[Vec<u8>] {
        &self.keys
    }

    /// Returns the physical key prefixes in this scope.
    #[must_use]
    pub fn prefixes(&self) -> &[Vec<u8>] {
        &self.prefixes
    }

    /// Checks whether a physical storage key matches this scope in O(|key|) time.
    #[must_use]
    pub fn matches_key(&self, key: &[u8]) -> bool {
        if self.keys.binary_search_by(|k| k.as_slice().cmp(key)).is_ok() {
            return true;
        }

        let idx = match self.prefixes.binary_search_by(|p| p.as_slice().cmp(key)) {
            Ok(_exact) => return true,
            Err(insertion_point) => insertion_point,
        };

        if idx > 0 && key.starts_with(self.prefixes[idx - 1].as_slice()) {
            return true;
        }

        false
    }

    /// Returns true if this physical scope contains no explicit keys and no prefixes.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty() && self.prefixes.is_empty()
    }

    /// Computes the canonical BLAKE3 scope digest using domain tag `contextra/physical-scope/v1` (§5.4).
    #[must_use]
    pub fn scope_digest(&self) -> [u8; 32] {
        let mut hasher = blake3::Hasher::new_derive_key("contextra/physical-scope/v1");

        hasher.update(&(self.keys.len() as u64).to_le_bytes());
        for key in &self.keys {
            hasher.update(&(key.len() as u64).to_le_bytes());
            hasher.update(key);
        }

        hasher.update(&(self.prefixes.len() as u64).to_le_bytes());
        for prefix in &self.prefixes {
            hasher.update(&(prefix.len() as u64).to_le_bytes());
            hasher.update(prefix);
        }

        *hasher.finalize().as_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_physical_scope_is_empty() {
        let empty_scope = PhysicalScope {
            keys: vec![],
            prefixes: vec![],
        };
        assert!(empty_scope.is_empty());

        let scope_with_key = PhysicalScope {
            keys: vec![b"t:1:k1".to_vec()],
            prefixes: vec![],
        };
        assert!(!scope_with_key.is_empty());

        let scope_with_prefix = PhysicalScope {
            keys: vec![],
            prefixes: vec![b"t:1:p1".to_vec()],
        };
        assert!(!scope_with_prefix.is_empty());
    }
}

/// Linear token representing an active erasure lifecycle invocation (§5.5).
///
/// Neither `Copy` nor `Clone`. Consumed by `finish` and `abort`.
#[must_use]
#[derive(Debug, PartialEq, Eq)]
pub struct HookToken {
    id: u64,
    _no_copy: (),
}

impl HookToken {
    /// Sentinel token representing no-op or inactive hooks.
    pub const NONE: Self = Self { id: 0, _no_copy: () };

    /// Creates a new `HookToken` wrapping raw numeric identifier.
    pub const fn new(id: u64) -> Self {
        Self { id, _no_copy: () }
    }

    /// Returns raw numeric ID.
    pub const fn id(&self) -> u64 {
        self.id
    }
}

/// Context passed when beginning an erasure lifecycle (§5.5).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EraseCtx<'a> {
    /// Target tenant ID.
    pub tenant_id: TenantId,
    /// Collection ID if collection or document scoped.
    pub collection_id: Option<CollectionId>,
    /// Document ID if document scoped.
    pub doc_id: Option<DocId>,
    /// Underlying logical scope.
    pub scope: &'a LogicalScope,
}

/// Active write fence guard held during purge execution.
pub struct FenceGuard {
    /// Identifier of purge holding the fence.
    pub purge_id: PurgeId,
    on_release: Option<Box<dyn FnOnce() + Send>>,
}

impl FenceGuard {
    /// Creates a new `FenceGuard` without a release callback.
    pub const fn new(purge_id: PurgeId) -> Self {
        Self {
            purge_id,
            on_release: None,
        }
    }

    /// Creates a new `FenceGuard` with a custom release callback called on drop.
    pub fn with_release(purge_id: PurgeId, on_release: impl FnOnce() + Send + 'static) -> Self {
        Self {
            purge_id,
            on_release: Some(Box::new(on_release)),
        }
    }
}

impl std::fmt::Debug for FenceGuard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FenceGuard")
            .field("purge_id", &self.purge_id)
            .field("has_release_callback", &self.on_release.is_some())
            .finish()
    }
}

impl Drop for FenceGuard {
    fn drop(&mut self) {
        if let Some(cb) = self.on_release.take() {
            cb();
        }
    }
}

/// Output report produced by an [`ErasureBackend::purge_physical`] operation (§5.5).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PurgeOutput {
    /// Number of records/versions physically dropped.
    pub records_purged: u64,
    /// Total disk bytes reclaimed.
    pub bytes_reclaimed: u64,
    /// Number of SSTable files rewritten/replaced.
    pub sstables_replaced: usize,
    /// Obsolete files identified for disposal.
    pub obsolete_files: Vec<PathBuf>,
    /// Execution duration in clock ticks.
    pub duration_ticks: u64,
}

/// Reasons why a purge compaction operation is blocked (precondition / temporary condition, safe to retry).
#[derive(Debug, Clone, PartialEq, Eq, Error, strum::EnumIter)]
pub enum PurgeBlocked {
    /// Active MVCC snapshot pins a sequence number <= as_of_tx.
    #[error("Active MVCC snapshot pins min_seq {min_seq}")]
    ActiveSnapshot { min_seq: u64 },
    /// Checkpoint clones present.
    #[error("Checkpoint clones present: {copy_ids:?}")]
    CheckpointClones { copy_ids: Vec<String> },
    /// Reader is pinned in active memtable / scan.
    #[error("Reader pinned in active memtable or scan")]
    ReaderPinned,
    /// Background compaction currently busy.
    #[error("Background compaction currently busy")]
    CompactionBusy,
    /// Purge operation already in progress.
    #[error("Purge operation in progress")]
    PurgeInProgress,
    /// Insufficient disk space to write compacted SSTables safely.
    #[error("Insufficient disk space: needed {needed}, free {free}")]
    InsufficientDiskSpace { needed: u64, free: u64 },
    /// Unclassified files present in DB directory.
    #[error("Unclassified files present in DB directory: {count}")]
    UnclassifiedFiles { count: usize },
    /// Secure deletion is unsupported on current filesystem/environment.
    #[error("Secure deletion unsupported: {reason}")]
    SecureDeleteUnsupported { reason: String },
    /// Unknown free space.
    #[error("Unknown free space")]
    UnknownFreeSpace,
}

impl Default for PurgeBlocked {
    fn default() -> Self {
        Self::ActiveSnapshot { min_seq: 0 }
    }
}

impl PurgeBlocked {
    /// Actionable retry recommendation for the caller.
    #[must_use]
    pub fn retry_hint(&self) -> &'static str {
        match self {
            Self::ActiveSnapshot { .. } => "Close open read transactions / snapshots",
            Self::CheckpointClones { .. } => "Purge or release registered checkpoint clones",
            Self::ReaderPinned => "Wait for active reader scan to finish",
            Self::CompactionBusy => "Retry after current background compaction completes",
            Self::PurgeInProgress => "Retry after active purge operation finishes",
            Self::InsufficientDiskSpace { .. } => "Free disk space or increase storage capacity",
            Self::UnclassifiedFiles { .. } => "Clean up untracked or stale files in storage directory",
            Self::SecureDeleteUnsupported { .. } => "Disable scrub requirement or run on supported filesystem",
            Self::UnknownFreeSpace => "Provide a FreeSpaceProbe implementation or ensure disk space is queryable",
        }
    }
}

/// Hard or blocked error during purge operations (§5.5).
#[derive(Debug, Error, strum::EnumIter)]
pub enum PurgeError {
    /// Operation blocked by precondition (can be retried).
    #[error("Purge blocked: {0}")]
    Blocked(#[from] PurgeBlocked),

    /// Read error during SSTable / storage processing.
    #[error("Read error at file {file}, offset {offset}: {source}")]
    Read {
        file: String,
        offset: u64,
        source: String,
    },

    /// Write error during SSTable / storage processing.
    #[error("Write error at file {file}: {source}")]
    Write {
        file: String,
        source: String,
    },

    /// Manifest log or state error.
    #[error("Manifest error: {0}")]
    Manifest(String),

    /// Pending delete journal error.
    #[error("Journal error: {0}")]
    Journal(String),

    /// Write fence conflict or acquisition failure.
    #[error("Fence error: {0}")]
    Fence(String),

    /// Corrupted or invalid purge intent.
    #[error("Intent error: {0}")]
    Intent(String),

    /// Scope unavailable during reconstruction or execution.
    #[error("Scope unavailable for purge_id {purge_id:?}")]
    ScopeUnavailable { purge_id: PurgeId },

    /// Scope mismatch between intent and expected scope.
    #[error("Scope mismatch for purge_id {purge_id:?}")]
    ScopeMismatch { purge_id: PurgeId },

    /// Key residue discovered after purge sweep.
    #[error("Residue found at file {file}")]
    ResidueFound { file: String },

    /// Disposal/scrubbing refused by policy or system constraint.
    #[error("Dispose refused: {0}")]
    DisposeRefused(String),

    /// Failpoint triggered during test execution.
    #[cfg(any(test, feature = "fault-injection"))]
    #[error("Failpoint triggered: {0}")]
    Failpoint(&'static str),
}

impl From<std::io::Error> for PurgeError {
    fn from(err: std::io::Error) -> Self {
        PurgeError::Write {
            file: "<io>".to_string(),
            source: err.to_string(),
        }
    }
}

impl From<PurgeError> for ContextraError {
    fn from(err: PurgeError) -> Self {
        ContextraError::Storage(format!("Purge error: {err}"))
    }
}

/// Storage backend contract for erasure operations (§5.5).
pub trait ErasureBackend: Send + Sync {
    /// Translates a logical scope into a tenant-encoded physical scope.
    fn physical_scope(&self, scope: &LogicalScope) -> std::result::Result<PhysicalScope, ContextraError>;

    /// Acquires write drain fence for `scope`.
    fn acquire_fence(&self, scope: &PhysicalScope, id: PurgeId) -> std::result::Result<FenceGuard, ContextraError>;

    /// Executes physical purge compaction for `scope` as of `as_of`.
    fn purge_physical(
        &self,
        scope: &PhysicalScope,
        as_of: TxId,
        probe: &dyn FreeSpaceProbe,
    ) -> std::result::Result<PurgeOutput, PurgeError>;
}

/// Finish artifact generated upon erasure completion.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FinishArtifact {
    /// No proof required (e.g. Assured mode).
    None,
    /// Attestation proof bytes (Attested mode).
    Proof(Vec<u8>),
}

/// Lifecycle hooks invoked by engine during erasure execution (§5.5).
pub trait ErasureHooks: Send + Sync + 'static {
    /// Invoked before commit point; appends `ErasureIntent` to ledger.
    fn on_begin(&self, ctx: &EraseCtx<'_>) -> std::result::Result<HookToken, ContextraError>;

    /// Invoked after tombstone commit in MVCC.
    fn on_tombstoned(&self, tok: &HookToken, tx: TxId) -> std::result::Result<(), ContextraError>;

    /// Invoked after physical purge completed.
    fn on_purged(&self, tok: &HookToken, out: &PurgeOutput) -> std::result::Result<(), ContextraError>;

    /// Finalizes erasure and yields finish artifact (consumes `HookToken`).
    fn finish(&self, tok: HookToken) -> std::result::Result<FinishArtifact, ContextraError>;

    /// Aborts erasure lifecycle (consumes `HookToken`).
    fn abort(&self, tok: HookToken);
}

/// Error returned when write operation hits an active write fence (§5.5).
#[derive(Debug, Error, PartialEq, Eq, Clone)]
#[error("Write operation fenced for key range")]
pub struct WriteFenced;

/// Write gate checking if mutations are permitted (§5.5).
pub trait WriteGate: Send + Sync {
    /// Fast-path check: returns true if any fence is active.
    fn is_active(&self) -> bool;

    /// Checks whether write batch is allowed.
    fn check_batch(&self, keys: &mut dyn Iterator<Item = &[u8]>) -> std::result::Result<(), WriteFenced>;
}

/// Error returned by free space probe (§5.5).
#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[error("Free space probe failed: {0}")]
pub struct ProbeError(pub String);

/// Disk space probe contract (§5.5).
pub trait FreeSpaceProbe: Send + Sync {
    /// Returns available disk space in bytes. None = unknown (never 0 bytes).
    fn available_bytes(&self) -> std::result::Result<Option<u64>, ProbeError>;

    /// Convenience helper returning available bytes as `Option<u64>`, ignoring probe errors.
    fn free_space_bytes(&self) -> Option<u64> {
        self.available_bytes().ok().flatten()
    }
}

/// Anti-resurrection guard verdict (§5.5).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RestoreVerdict {
    /// Key is admitted for restore.
    Allow,
    /// Key was erased and must be rejected.
    Reject,
}

/// Anti-resurrection guard preventing restored snapshots from resurrecting erased keys (§5.5).
pub trait RestoreGuard: Send + Sync {
    /// Checks whether `key` is permitted to be restored from snapshot at `source_tx`.
    fn admit(&self, key: &[u8], source_tx: TxId) -> RestoreVerdict;
}

/// Scan error for layer scan (§5.5).
#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[error("Scan error on layer {layer_id}: {reason}")]
pub struct ScanError {
    /// Layer identifier.
    pub layer_id: u16,
    /// Reason string.
    pub reason: String,
}

/// Layer observation evidence produced by [`LayerScan`] (§5.5).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LayerObservation {
    /// Numeric identifier of scanned layer (u16).
    pub layer_id: u16,
    /// Whether layer is clean (empty of scoped keys).
    pub clean: bool,
    /// BLAKE3 digest of scanned layer state.
    pub state_digest: [u8; 32],
    /// Number of items scanned (proof against empty scans).
    pub item_count: u64,
}

/// Scanner contract implemented by layer owners (§5.5).
pub trait LayerScan: Send + Sync {
    /// Numeric layer identifier (u16).
    fn layer_id(&self) -> u16;

    /// Scans layer for presence of keys in `scope`.
    fn scan(&self, scope: &PhysicalScope) -> std::result::Result<LayerObservation, ScanError>;
}

/// Outcome of a deletion request (§5.3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeleteOutcome {
    /// Mode Off: Tombstone committed.
    Logical {
        /// Transaction ID of committed tombstone.
        tx: TxId,
    },
    /// Mode Assured: Ledger durable, tombstone committed, purge scheduled or running.
    Assured {
        /// Transaction ID of committed tombstone.
        tx: TxId,
        /// Purge operation handle.
        purge_id: PurgeId,
    },
}

/// Error returned by erasure operations (§5.3).
#[derive(Debug, Error)]
pub enum EraseError {
    /// Failed before commit point (retry is safe).
    #[error("Deletion failed before commit point: {0}")]
    NotStarted(#[source] ContextraError),

    /// State indeterminate after commit point (deletion will roll forward).
    #[error("Deletion state indeterminate after commit point (purge_id: {purge_id:?}): {cause}")]
    Indeterminate {
        /// Identifier of purge operation.
        purge_id: PurgeId,
        /// Cause of failure after commit point.
        #[source]
        cause: ContextraError,
    },
}

/// Purge operation phase state machine (§5.3, §5.6, E67).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum PurgeState {
    /// Step 0: hooks.begin() invoked.
    Begun = 0,
    /// Step 1: Ledger record fsync-ed (commit point).
    LedgerAppended = 1,
    /// Step 2: Tombstone committed.
    Tombstoned = 2,
    /// Step 3: Pending purge execution.
    PurgePending = 3,
    /// Step 4: Fence held, rewrite active.
    Purging = 4,
    /// Step 5: Rewrite, scrub, and derived stores complete.
    Purged = 5,
    /// Step 6: Crypto-erase stage 1 complete (§5.13).
    KeyQuarantined = 6,
    /// Step 7: Assured phase complete (EP1 terminal state).
    Completed = 7,
    /// Step 8: Scans and proofs complete (EP2).
    Attested = 8,
}

impl PurgeState {
    /// Converts PurgeState to u8 representation.
    #[must_use]
    pub const fn to_u8(self) -> u8 {
        self as u8
    }

    /// Parses PurgeState from u8 code.
    #[must_use]
    pub fn from_u8(val: u8) -> Option<Self> {
        match val {
            0 => Some(Self::Begun),
            1 => Some(Self::LedgerAppended),
            2 => Some(Self::Tombstoned),
            3 => Some(Self::PurgePending),
            4 => Some(Self::Purging),
            5 => Some(Self::Purged),
            6 => Some(Self::KeyQuarantined),
            7 => Some(Self::Completed),
            8 => Some(Self::Attested),
            _ => None,
        }
    }
}
