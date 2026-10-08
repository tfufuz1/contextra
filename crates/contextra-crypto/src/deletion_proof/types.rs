//! Domain types and hashing helpers for deletion proofs.

use contextra_types::{CollectionId, DocId, TenantId};
use serde::{Deserialize, Serialize};

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
}

/// Explizite Nicht-Abdeckung — maschinenlesbar für Audit-Systeme.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExcludedScope {
    /// Wissen in Zusammenfassungen die als LLM-Fine-Tuning-Input dienten.
    ConsolidatedAndDistilled,
    /// LLM-Modellparameter (arXiv:2505.16831 — Unlearning Isn't Deletion).
    LlmParameterMemory,
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
