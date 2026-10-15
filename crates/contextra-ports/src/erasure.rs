//! Erasure and deletion guarantee port traits and types (§8.3, §8.4, ADR-01, ADR-02, ADR-14).
//!
//! Provides the abstract contracts for multi-tier document erasure:
//! - [`ErasureMode`]: System-wide erasure watermark (`Off`, `Assured`, `Attested`).
//! - [`ErasureBackend`]: Storage-level physical scope derivation and purge execution.
//! - [`ErasureHooks`]: Lifecycle callbacks during coordinator execution.
//! - [`LayerScan`]: Verification scan contracts for storage/index layers.
//! - [`RestoreGuard`]: Anti-resurrection guard for checkpoint restoration.
//! - [`PhysicalScope`]: Concrete tenant-encoded physical keys and prefixes.

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::BoxFuture;
use contextra_types::{CollectionId, DocId, Result, TenantId, TxId};

/// Operational mode for document erasure and deletion guarantees (§8.2).
///
/// Persistent watermark: opening with a lower mode than previously recorded
/// fails closed with a policy violation.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default,
)]
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

/// Logical scope of a deletion request before physical tenant encoding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogicalScope {
    /// A single document within a tenant.
    Document {
        /// Target document identifier.
        doc_id: DocId,
        /// Tenant owning the document.
        tenant_id: TenantId,
    },
    /// An entire collection within a tenant.
    Collection {
        /// Target collection identifier.
        collection_id: CollectionId,
        /// Tenant owning the collection.
        tenant_id: TenantId,
    },
    /// Explicit list of logical key bytes.
    Keys(Vec<Vec<u8>>),
    /// List of logical key prefix bytes.
    Prefixes(Vec<Vec<u8>>),
}

/// Physical, tenant-encoded keys and prefixes (§8.4, ADR-14).
///
/// Structural invariant: `purge_keys` only accepts `PhysicalScope` to prevent
/// prefix-mismatch bugs (N-K) where logical prefixes are passed to raw storage.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PhysicalScope {
    /// Explicit physical keys.
    pub keys: Vec<Vec<u8>>,
    /// Physical key prefixes.
    pub prefixes: Vec<Vec<u8>>,
}

impl PhysicalScope {
    /// Creates a new `PhysicalScope` from explicit keys and prefixes.
    #[must_use]
    pub fn new(keys: Vec<Vec<u8>>, prefixes: Vec<Vec<u8>>) -> Self {
        Self { keys, prefixes }
    }

    /// Checks if a physical storage key matches this scope.
    #[must_use]
    pub fn matches_key(&self, key: &[u8]) -> bool {
        self.keys.iter().any(|k| k.as_slice() == key)
            || self.prefixes.iter().any(|p| key.starts_with(p.as_slice()))
    }

    /// Returns true if the scope contains no keys and no prefixes.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty() && self.prefixes.is_empty()
    }
}

/// Output report produced by an [`ErasureBackend::purge`] operation (§8.5).
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
}

/// Storage backend contract for erasure operations (§8.3, ADR-01).
///
/// Replaces downcasts to concrete storage types (`LsmStorage`).
pub trait ErasureBackend: Send + Sync {
    /// Translates a logical scope into a tenant-encoded physical scope.
    fn physical_scope(&self, scope: &LogicalScope) -> Result<PhysicalScope>;

    /// Executes physical purge compaction for `scope` as of `as_of_tx`.
    fn purge<'a>(
        &'a self,
        scope: &'a PhysicalScope,
        as_of: TxId,
    ) -> BoxFuture<'a, Result<PurgeOutput>>;
}

/// Context information passed when beginning an erasure lifecycle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EraseCtx {
    /// Target document ID if document-scoped.
    pub doc_id: Option<DocId>,
    /// Target tenant ID.
    pub tenant_id: TenantId,
    /// Collection name if applicable.
    pub collection: Option<String>,
}

/// Token identifying an active erasure lifecycle invocation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct HookToken(pub u64);

impl HookToken {
    /// Sentinel token representing no-op or inactive hooks.
    pub const NONE: Self = Self(0);
}

/// Evidence observation produced by a [`LayerScan`] (§8.4).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LayerObservation {
    /// Numeric identifier of the scanned layer.
    pub layer_id: u8,
    /// Whether the layer was observed empty of scoped keys.
    pub observed_empty: bool,
    /// Human-readable observation details.
    pub details: String,
}

/// Attestation produced upon completion of an attested erasure lifecycle (§8.4).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErasureAttestation {
    /// Proof identifier (BLAKE3 / SHA-256 digest).
    pub proof_id: [u8; 32],
    /// Mode under which the attestation was generated.
    pub mode: ErasureMode,
    /// Timestamp of attestation.
    pub timestamp_ms: u64,
}

/// Scanner contract implemented by layer owners (Store, Vector, Text, Graph) (§8.3).
pub trait LayerScan: Send + Sync {
    /// Numeric layer identifier.
    fn layer_id(&self) -> u8;

    /// Scans the layer for presence of keys in `scope`. Must fail closed on error.
    fn scan(&self, scope: &PhysicalScope) -> Result<LayerObservation>;
}

/// Lifecycle hooks invoked by the engine during erasure execution (§8.4).
pub trait ErasureHooks: Send + Sync + 'static {
    /// Invoked before locking and footprint computation.
    fn begin(&self, _ctx: &EraseCtx) -> Result<HookToken> {
        Ok(HookToken::NONE)
    }

    /// Invoked immediately after tombstones have been committed.
    fn tombstoned(&self, _token: &HookToken, _tx: TxId) -> Result<()> {
        Ok(())
    }

    /// Invoked after physical purge has been executed.
    fn purged(&self, _token: &HookToken, _scans: &[&dyn LayerScan]) -> Result<()> {
        Ok(())
    }

    /// Finalizes erasure and produces an attestation if in Attested mode.
    fn finish(&self, token: HookToken) -> Result<Option<ErasureAttestation>>;
}

/// Verdict returned by a [`RestoreGuard`] when filtering restored keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RestoreVerdict {
    /// Key is admitted for restore.
    Allow,
    /// Key was erased and must be rejected (anti-resurrection).
    Reject,
}

/// Anti-resurrection guard preventing restored snapshots from resurrecting erased keys (§8.7).
pub trait RestoreGuard: Send + Sync {
    /// Checks whether `key` is allowed to be restored from checkpoint at `cp_tx`.
    fn admit(&self, key: &[u8], cp_tx: TxId) -> RestoreVerdict;
}

/// Gate controlling whether writes are permitted into protected key ranges (§8.6).
pub trait WriteGate: Send + Sync {
    /// Returns true if writes to `key` are currently permitted.
    fn check_allowed(&self, key: &[u8]) -> bool;
}
