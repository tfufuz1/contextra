//! Erasure and deletion guarantee port traits and types (§5.3, §5.4, §5.5, §8.3, §8.4).
//!
//! Provides abstract contracts for multi-tier document erasure:
//! - [`ErasureMode`]: System-wide erasure watermark (`Off`, `Assured`, `Attested`).
//! - [`ErasureBackend`]: Storage-level physical scope derivation and purge execution.
//! - [`ErasureHooks`]: Lifecycle callbacks during coordinator execution.
//! - [`LayerScan`]: Verification scan contracts for storage/index layers.
//! - [`LayerEraser`]: Storage layer physical data removal contracts.
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PurgeId(pub [u8; 16]);

impl PurgeId {
    /// Creates a new `PurgeId` wrapping raw 16-byte identifier.
    pub const fn new(bytes: [u8; 16]) -> Self {
        Self(bytes)
    }
}

/// Numeric storage or index layer identifier (§5.5).
pub type LayerId = u16;

/// Scope provenance and origin metadata (§4.3, DE-03).
///
/// Verdrahtung: J-20
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct ScopeOrigin {
    /// Random process/instance-stable storage backend identifier.
    pub backend_id: u64,
    /// Tenant BLAKE3 digest.
    pub tenant_digest: [u8; 32],
}

impl ScopeOrigin {
    /// Creates a new `ScopeOrigin`.
    pub fn new(backend_id: u64, tenant_digest: [u8; 32]) -> Self {
        Self {
            backend_id,
            tenant_digest,
        }
    }
}

/// Error returned by erasure backend operations (§4.3).
#[derive(Debug, Error, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErasureError {
    /// Erasure backend not available.
    #[error("Erasure backend not available")]
    NotAvailable,
    /// Policy violation.
    #[error("Policy violation: current {current}, requested {requested}")]
    PolicyViolation {
        /// Currently active policy.
        current: String,
        /// Requested policy.
        requested: String,
    },
    /// Erasure backend not configured.
    #[error("Erasure backend not configured")]
    NotConfigured,
    /// Scope belongs to a foreign tenant or mismatched backend origin.
    #[error("Foreign scope or backend origin mismatch")]
    ForeignScope,
    /// Scope key or prefix violates tenant encoding invariants.
    #[error("Malformed physical scope")]
    MalformedScope,
    /// Erasure ledger state is corrupt.
    #[error("Erasure ledger corrupt")]
    LedgerCorrupt,
    /// Internal failure in erasure subsystem.
    #[error("Internal erasure error: {0}")]
    Internal(String),
}

impl From<ErasureError> for ContextraError {
    fn from(err: ErasureError) -> Self {
        match err {
            ErasureError::NotAvailable => ContextraError::capability_unsupported(
                "erasure_backend",
                "Erasure backend not available on this storage engine",
            ),
            ErasureError::PolicyViolation { current, requested } => ContextraError::PolicyViolation(
                format!("Erasure policy violation: current={}, requested={}", current, requested),
            ),
            ErasureError::NotConfigured => ContextraError::InvalidInput(
                "Erasure backend not configured".to_string(),
            ),
            ErasureError::ForeignScope => ContextraError::InvalidInput(
                "Foreign physical scope or backend origin mismatch".to_string(),
            ),
            ErasureError::MalformedScope => ContextraError::InvalidInput(
                "Malformed physical scope: key/prefix does not match tenant prefix or format rules".to_string(),
            ),
            ErasureError::LedgerCorrupt => ContextraError::Storage(
                "Erasure ledger corrupt".to_string(),
            ),
            ErasureError::Internal(msg) => ContextraError::Storage(msg),
        }
    }
}

/// Obsolete file record identified for disposal (§4.3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObsoleteFile {
    /// File path.
    pub path: PathBuf,
    /// Byte length of file.
    pub byte_len: u64,
}

/// Report returned by [`ErasureBackend::dispose`] (§4.3).
///
/// Verdrahtung: J-20
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct DisposeReport {
    /// Number of files removed.
    pub files_removed: usize,
    /// Total disk space freed in bytes.
    pub bytes_freed: u64,
}

/// Error returned by [`ErasureBackend::dispose`] (§4.3).
///
/// Verdrahtung: J-20
#[derive(Debug, Error, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DisposeError {
    /// File scrub was refused by storage or policy.
    #[error("Scrub refused for file {file}: {reason}")]
    ScrubRefused {
        /// Target file path.
        file: String,
        /// Refusal reason.
        reason: String,
    },
    /// File removal failed.
    #[error("Failed to remove file: {file}")]
    RemoveFailed {
        /// Target file path.
        file: String,
    },
    /// Filesystem sync failed after disposal.
    #[error("Sync failed during file disposal")]
    SyncFailed,
}

/// Error returned by purge resume operation (§4.3).
#[derive(Debug, Error, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResumeError {
    /// Target purge ID was not recorded in ledger.
    #[error("Scope not in ledger for purge_id: {purge_id:?}")]
    ScopeNotInLedger {
        /// Purge operation handle.
        purge_id: PurgeId,
    },
    /// Purge intent file or record is corrupt.
    #[error("Purge intent corrupt")]
    IntentCorrupt,
    /// Erasure ledger state is corrupt.
    #[error("Erasure ledger corrupt")]
    LedgerCorrupt,
    /// Storage engine not ready for resume.
    #[error("Store not ready")]
    StoreNotReady,
}

/// Error returned by checkpoint restore verification (§4.3).
#[derive(Debug, Error, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RestoreError {
    /// Anti-resurrection restore guard is required for active erasure mode.
    #[error("Restore guard required for mode: {mode}")]
    GuardRequired {
        /// Active erasure mode.
        mode: String,
    },
    /// Erasure ledger state is corrupt.
    #[error("Erasure ledger corrupt")]
    LedgerCorrupt,
    /// I/O error during restore operation.
    #[error("I/O error during restore: {0}")]
    Io(String),
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
    origin: ScopeOrigin,
    keys: Vec<Vec<u8>>,
    prefixes: Vec<Vec<u8>>,
}

impl PhysicalScope {
    /// Creates a new validated, canonical `PhysicalScope` (§5.4).
    ///
    /// # Invariants
    /// 1. Must not be empty (at least one key or prefix required).
    /// 2. Every key and prefix MUST start with `t:<tenant_id>:`.
    /// 3. Every prefix MUST end with `:`.
    /// 4. Canonical: sorted, deduplicated, and subsumed keys/prefixes removed.
    #[cfg(feature = "backend-impl")]
    pub fn try_new(
        tenant_id: TenantId,
        origin: ScopeOrigin,
        keys: Vec<Vec<u8>>,
        prefixes: Vec<Vec<u8>>,
    ) -> std::result::Result<Self, ErasureError> {
        if keys.is_empty() && prefixes.is_empty() {
            return Err(ErasureError::MalformedScope);
        }

        let tenant_prefix_bytes = format!("t:{}:", tenant_id.0).into_bytes();

        for k in &keys {
            if !k.starts_with(&tenant_prefix_bytes) {
                return Err(ErasureError::MalformedScope);
            }
        }

        for p in &prefixes {
            if p.is_empty() || !p.starts_with(&tenant_prefix_bytes) || !p.ends_with(b":") {
                return Err(ErasureError::MalformedScope);
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
            origin,
            keys: canonical_keys,
            prefixes: canonical_prefixes,
        })
    }

    /// Returns the provenance origin of this physical scope.
    #[must_use]
    pub fn origin(&self) -> &ScopeOrigin {
        &self.origin
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

    /// Returns false since a valid `PhysicalScope` is guaranteed to be non-empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        false
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
    purge_id: PurgeId,
    release: Option<Box<dyn FnOnce() + Send + Sync>>,
}

impl std::fmt::Debug for FenceGuard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FenceGuard")
            .field("purge_id", &self.purge_id)
            .finish()
    }
}

impl FenceGuard {
    /// Creates a new `FenceGuard` with a purge ID and a drop release hook.
    pub fn new(purge_id: PurgeId, release: Box<dyn FnOnce() + Send + Sync>) -> Self {
        Self {
            purge_id,
            release: Some(release),
        }
    }

    /// Returns the identifier of the purge operation holding the fence.
    #[must_use]
    pub fn purge_id(&self) -> PurgeId {
        self.purge_id
    }
}

impl Drop for FenceGuard {
    fn drop(&mut self) {
        if let Some(release) = self.release.take() {
            release();
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

/// Error during purge compaction (§5.5).
#[derive(Debug, Error)]
pub enum PurgeError {
    /// Insufficient free disk space to write rewritten SSTable segments.
    #[error("Insufficient disk space: needed {needed} bytes, available {available:?}")]
    InsufficientDiskSpace {
        /// Disk space needed in bytes.
        needed: u64,
        /// Available disk space in bytes (None = probe unknown).
        available: Option<u64>,
    },
    /// Write fence conflict detected during purge execution.
    #[error("Fence conflict")]
    FenceConflict,
    /// Read error during SSTable segment processing.
    #[error("Read error at file {file}, offset {offset}: {source_details}")]
    ReadError {
        /// Segment path.
        file: String,
        /// Offset in file.
        offset: u64,
        /// Error detail string.
        source_details: String,
    },
    /// Write error during SSTable segment or temporary file writing.
    #[error("Write error at file {file}: {source_details}")]
    WriteError {
        /// Target file path.
        file: String,
        /// Error detail string.
        source_details: String,
    },
    /// Background compaction lease is currently busy.
    #[error("Compaction busy")]
    CompactionBusy,
    /// I/O error wrapper.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    /// Manifest log processing error.
    #[error("Manifest error: {0}")]
    Manifest(String),
    /// Flush operation failed prior to purge execution.
    #[error("Flush failed: {0}")]
    FlushFailed(String),
    /// Free space probe failed.
    #[error("Probe failed: {0}")]
    ProbeFailed(String),
    /// Long-lived snapshot pin prevents physical deletion.
    #[error("Snapshot pinned: oldest pin age {oldest_pin_age} ticks")]
    SnapshotPinned {
        /// Oldest snapshot pin age in clock ticks.
        oldest_pin_age: u64,
    },
}

/// Storage backend contract for erasure operations (§4.3, §5.5).
pub trait ErasureBackend: Send + Sync {
    /// Translates a logical scope into a tenant-encoded physical scope.
    fn physical_scope(&self, scope: &LogicalScope) -> std::result::Result<PhysicalScope, ErasureError>;

    /// Acquires write drain fence for `scope`.
    fn acquire_fence(&self, scope: &PhysicalScope, id: PurgeId) -> std::result::Result<FenceGuard, ErasureError>;

    /// Executes physical purge compaction for `scope` as of `as_of`.
    fn purge_physical(
        &self,
        scope: &PhysicalScope,
        as_of: TxId,
        probe: &dyn FreeSpaceProbe,
        fence: &FenceGuard,
    ) -> std::result::Result<PurgeOutput, PurgeError>;

    /// Disposes obsolete files (§4.3).
    fn dispose(&self, _files: &[ObsoleteFile]) -> std::result::Result<DisposeReport, DisposeError> {
        Err(DisposeError::ScrubRefused {
            file: String::new(),
            reason: "Dispose operation not available".to_string(),
        })
    }

    /// Returns the erasure fence snapshot write sequence number for the specified prefixes.
    fn snapshot_write_seq(&self, _prefixes: &[Vec<u8>]) -> u64 {
        0
    }
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

/// Report returned by layer erasure execution (§4.3).
///
/// Verdrahtung: J-20
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct EraseReport {
    /// Numeric layer identifier.
    pub layer: LayerId,
    /// Number of items erased in layer.
    pub items_erased: u64,
    /// Bytes scrubbed/reclaimed in layer.
    pub bytes_scrubbed: u64,
}

/// Layer eraser contract implemented by layer owners (idempotent, §4.3).
///
/// Implementations MUST be idempotent.
/// Verdrahtung: J-20
pub trait LayerEraser: Send + Sync {
    /// Layer identifier.
    fn layer(&self) -> LayerId;

    /// Erases scoped data from layer up to `as_of` transaction (idempotent).
    fn erase(&self, scope: &PhysicalScope, as_of: TxId) -> std::result::Result<EraseReport, ErasureError>;
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
    /// Mode Attested: Logical and assured processing complete, attestation proof reference issued (DE-21).
    Attested {
        /// Proof reference identifier.
        proof_ref: String,
    },
}

/// Error returned by erasure operations (§5.3).
#[derive(Debug, Error)]
pub enum EraseError {
    /// Erasure backend or capability not available on storage engine.
    #[error("Erasure backend not available")]
    NotAvailable,

    /// Failed before commit point (retry is safe).
    #[error("Deletion failed before commit point: {0}")]
    NotStarted(#[source] ContextraError),
}

impl From<EraseError> for ContextraError {
    fn from(err: EraseError) -> Self {
        match err {
            EraseError::NotAvailable => ContextraError::capability_unsupported(
                "erasure_backend",
                "Erasure backend not available on this storage engine",
            ),
            EraseError::NotStarted(cause) => cause,
        }
    }
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

/// Reasons why purge execution is currently pending (§4.3, §5.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum PendingReason {
    /// Active open read snapshot pins purge horizon.
    OpenSnapshot,
    /// Disk space is low or free space probe failed.
    DiskSpaceUnknownOrLow,
    /// File scrub operation was refused.
    ScrubRefused,
    /// Background compaction lease is busy.
    CompactionLease,
    /// Write fence active on requested key scope.
    Fenced,
    /// Read error encountered during execution.
    ReadError,
    /// Write error encountered during execution.
    WriteError,
    /// Manifest error encountered during execution.
    ManifestError,
    /// Key retention wait period active.
    KeyWaitPeriod,
}

/// Public purge status derived from internal `PurgeState` state machine (§4.3, §5.2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PurgeStatus {
    /// Purge execution pending.
    Pending {
        /// Reason for pending status.
        reason: PendingReason,
        /// Timestamp in milliseconds when purge entered pending state.
        since_ms: u64,
        /// Next retry timestamp in milliseconds.
        retry_at_ms: u64,
    },
    /// Purge rewrite in progress.
    InProgress {
        /// Number of SSTables processed.
        sstables_done: usize,
        /// Total number of SSTables to process.
        sstables_total: usize,
    },
    /// Physically purged.
    Purged,
    /// Assured phase completed.
    Completed,
    /// Witness scan failed for layer.
    ScanFailed {
        /// Numeric layer identifier.
        layer: u16,
    },
    /// Attestation proof available.
    Attested {
        /// Proof reference identifier.
        proof_ref: String,
    },
}

impl PurgeStatus {
    /// Derives public [`PurgeStatus`] from internal [`PurgeState`] (§5.2).
    /// Returns `None` if internal state is `Begun` (invisible state).
    #[must_use]
    pub fn from_state(state: PurgeState) -> Option<Self> {
        match state {
            PurgeState::Begun => None,
            PurgeState::LedgerAppended | PurgeState::Tombstoned | PurgeState::PurgePending => {
                Some(Self::Pending {
                    reason: PendingReason::KeyWaitPeriod,
                    since_ms: 0,
                    retry_at_ms: 0,
                })
            }
            PurgeState::Purging => Some(Self::InProgress {
                sstables_done: 0,
                sstables_total: 0,
            }),
            PurgeState::Purged | PurgeState::KeyQuarantined => Some(Self::Purged),
            PurgeState::Completed => Some(Self::Completed),
            PurgeState::Attested => Some(Self::Attested {
                proof_ref: String::new(),
            }),
        }
    }
}
