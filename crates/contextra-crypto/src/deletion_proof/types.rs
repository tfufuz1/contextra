//! Domain types and hashing helpers for deletion proofs.

use contextra_types::{CollectionId, DocId, TenantId};
use serde::{Deserialize, Serialize};

/// Attestation confirming that physical layer sanitization and deletion logs have been flushed to durable storage.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DurabilityProof {
    /// Monotonically increasing sequence number or TxId confirmed durable by disk fsync.
    pub confirmed_seq_no: u64,
}

impl DurabilityProof {
    /// Constructs a new DurabilityProof for a confirmed fsynced sequence number.
    pub fn new(confirmed_seq_no: u64) -> Self {
        Self { confirmed_seq_no }
    }
}

/// Attestation confirming synchronous neighborhood graph repair for vector deletions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GraphRepairAttestation {
    /// Document ID whose neighborhood pointers were repaired.
    pub doc_id: DocId,
    /// Verified confirmation that no ghost pointers remain.
    pub verified_no_ghost_pointers: bool,
    /// Unix timestamp when the repair was attested.
    pub attested_at: i64,
}

/// Berechnet den Blake3-Hash einer deterministisch sortierten Liste gelöschter Schlüssel mit Längenpräfix.
///
/// DOKUMENTATION ZUM KOLLISIONSRISIKO:
/// Ohne Längenpräfix pro Element (z. B. bloße Konkatenation) erzeugen unterschiedliche Key-Listen
/// wie `["ab", "c"]` und `["a", "bc"]` denselben Hash-Wert, da die Elementgrenzen im Byte-Strom
/// nicht kodiert sind. Durch das Voranstellen der Schlüssellänge als 4-Byte Little-Endian integer
/// (`(key.len() as u32).to_le_bytes()`) vor jedem Schlüssel-Byte-Array wird eine eindeutige,
/// kollisionsfreie Kodierung für jede Sequenz von Schlüssel-Bytes garantiert.
pub fn hash_deleted_keys_length_prefixed(deleted_keys: &[Vec<u8>]) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    for key in deleted_keys {
        let len_u32 = u32::try_from(key.len()).unwrap_or(u32::MAX);
        hasher.update(&len_u32.to_le_bytes());
        hasher.update(key);
    }
    *hasher.finalize().as_bytes()
}

/// Layer-explizite Coverage-Deklaration.
/// Jeder Layer MUSS physisch bereinigt sein bevor er hier deklariert wird.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DeletionLayer {
    /// In-memory memtable records cleared.
    LsmMemtable,
    /// Persistent SSTable files purged across all LSM levels.
    SsTableAllLevels,
    /// HNSW graph nodes and tombstone references purged.
    HnswIndex,
    /// WAL log segments truncated/zeroized.
    WalAllSegments {
        /// Sequence number after which WAL truncation occurred.
        seq_after: u64,
    },
    /// Compressed Sparse Row knowledge graph edges purged.
    CsrGraph,
    /// Key-value cache entries invalidated.
    KvCacheSegments,
    /// In-memory vector embedding cache cleared.
    EmbeddingCache,
    /// Text / BM25 search index postings and dictionaries cleared.
    TextIndex,
    /// Relational table and foreign-key link structures cleared.
    Relations,
    /// Database manifest and level metadata records updated/purged.
    Manifest,
    /// Checkpoint and snapshot clone references released and purged.
    CheckpointClones,
    /// Ephemeral process level caches and volatile buffers invalidated.
    ProcessCaches,
    /// Keyring cryptographic key wrappers and key material revoked/zeroized.
    Keyring,
    /// Erasure coding parity blocks and ledger entries purged.
    ErasureLedger,
}

/// Explizite Nicht-Abdeckung — maschinenlesbar für Audit-Systeme.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExcludedScope {
    /// Wissen in Zusammenfassungen die als LLM-Fine-Tuning-Input dienten.
    ConsolidatedAndDistilled,
    /// LLM-Modellparameter (arXiv:2505.16831 — Unlearning Isn't Deletion).
    LlmParameterMemory,
    /// Storage media physical remanence on magnetic or solid-state drives.
    StorageMediaRemanence,
    /// Unregistered or air-gapped cold backup archives.
    UnregisteredBackups,
    /// OS virtual memory page cache, swap files, or process crash dumps.
    OsPageCacheSwapCoreDumps,
    /// Temporary client session and active LLM model context buffers.
    ClientAndModelContext,
    /// Aggregated and anonymized statistical telemetry or metrics.
    AggregateDerivedStatistics,
    /// System log files and telemetry stored outside the database process.
    LogsAndTelemetryOutsideProcess,
    /// Data exported or transmitted to external third-party consumers.
    ExportedData,
    /// Explicitly registered copy identifiers that were not part of this deletion run.
    RegisteredCopies {
        /// Unique identifiers of excluded copies.
        ids: Vec<String>,
    },
    /// Persisted HNSW graph snapshots stored on disk.
    PersistedHnswSnapshots,
}

impl ExcludedScope {
    /// Returns the canonical u16 numerical identifier for the excluded scope.
    pub fn id(&self) -> u16 {
        match self {
            Self::ConsolidatedAndDistilled => 1,
            Self::LlmParameterMemory => 2,
            Self::StorageMediaRemanence => 3,
            Self::UnregisteredBackups => 4,
            Self::OsPageCacheSwapCoreDumps => 5,
            Self::ClientAndModelContext => 6,
            Self::AggregateDerivedStatistics => 7,
            Self::LogsAndTelemetryOutsideProcess => 8,
            Self::ExportedData => 9,
            Self::RegisteredCopies { .. } => 10,
            Self::PersistedHnswSnapshots => 11,
        }
    }

    /// Constructs an `ExcludedScope` from its numerical identifier.
    ///
    /// Note: `RegisteredCopies` (ID 10) returns `None` because the inner `ids: Vec<String>`
    /// payload cannot be derived solely from its numerical ID.
    pub fn from_id(id: u16) -> Option<Self> {
        match id {
            1 => Some(Self::ConsolidatedAndDistilled),
            2 => Some(Self::LlmParameterMemory),
            3 => Some(Self::StorageMediaRemanence),
            4 => Some(Self::UnregisteredBackups),
            5 => Some(Self::OsPageCacheSwapCoreDumps),
            6 => Some(Self::ClientAndModelContext),
            7 => Some(Self::AggregateDerivedStatistics),
            8 => Some(Self::LogsAndTelemetryOutsideProcess),
            9 => Some(Self::ExportedData),
            10 => None,
            11 => Some(Self::PersistedHnswSnapshots),
            _ => None,
        }
    }
}

/// Target scope for physical deletion.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DeletionScope {
    /// Individual document deletion.
    Document {
        /// DocId of target document.
        doc_id: DocId,
        /// TenantId of owning tenant.
        tenant_id: TenantId,
    },
    /// Collection-wide deletion.
    Collection {
        /// CollectionId of target collection.
        collection_id: CollectionId,
        /// TenantId of owning tenant.
        tenant_id: TenantId,
    },
    /// Tenant-wide deletion.
    Tenant {
        /// TenantId of target tenant.
        tenant_id: TenantId,
    },
}
