// FILE-CONTEXT
// ZWECK: Structural types and data definitions for cryptographic deletion proofs (GDPR Art. 17).
// INVARIANTEN: INV-DELETION-1: Proof creation requires verified layer sanitization witnesses.

#![forbid(unsafe_code)]

use contextra_ports::scanner::{LayerWitness, ScanTechnique, WitnessLayer};
use contextra_types::{CollectionId, ContextraError, DocId, Result, TenantId, TxId};
use serde::{Deserialize, Serialize};

/// Extension trait providing accessors for `LayerWitness` without mutating `contextra-ports`.
pub trait LayerWitnessExt {
    /// Extracts the `wal_seq_after` if layer is `WalAllSegments`, or `None`.
    fn wal_seq_after(&self) -> Option<u64>;
    /// Extracts the positive control check match count.
    fn control_matches(&self) -> u64;
}

impl LayerWitnessExt for LayerWitness {
    fn wal_seq_after(&self) -> Option<u64> {
        if let WitnessLayer::WalAllSegments = self.witness_layer() {
            Some(self.measured_at())
        } else {
            None
        }
    }

    fn control_matches(&self) -> u64 {
        if matches!(self.method(), contextra_types::ScanMethod::FeatureDisabled) {
            0
        } else {
            1
        }
    }
}

/// Mappt `WitnessLayer` erschöpfend auf `DeletionLayer` ohne `_`-Zweig.
pub fn map_witness_layer_to_deletion_layer(
    layer: &WitnessLayer,
    witness: &LayerWitness,
) -> Result<DeletionLayer> {
    match layer {
        WitnessLayer::LsmMemtable => Ok(DeletionLayer::LsmMemtable),
        WitnessLayer::SsTableAllLevels => Ok(DeletionLayer::SsTableAllLevels),
        WitnessLayer::HnswIndex => Ok(DeletionLayer::HnswIndex),
        WitnessLayer::WalAllSegments => {
            let seq_after = witness.wal_seq_after().ok_or_else(|| {
                ContextraError::Internal(
                    "WalAllSegments witness missing mandatory wal_seq_after sequence number"
                        .to_string(),
                )
            })?;
            Ok(DeletionLayer::WalAllSegments { seq_after })
        }
        WitnessLayer::CsrGraph => Ok(DeletionLayer::CsrGraph),
        WitnessLayer::KvCacheSegments => Ok(DeletionLayer::KvCacheSegments),
        WitnessLayer::EmbeddingCache => Ok(DeletionLayer::EmbeddingCache),
        WitnessLayer::Custom(s) => match s.as_str() {
            "TextIndex" => Ok(DeletionLayer::TextIndex),
            other => Err(ContextraError::Internal(format!(
                "Unrecognized custom WitnessLayer: '{other}' cannot be mapped to DeletionLayer"
            ))),
        },
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
pub fn hash_deleted_keys_length_prefixed(deleted_keys: &[Vec<u8>]) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    for key in deleted_keys {
        hasher.update(&(key.len() as u32).to_le_bytes());
        hasher.update(key);
    }
    *hasher.finalize().as_bytes()
}

/// Custody level for cryptographic key management during deletion proof generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CustodyLevel {
    /// Local custody.
    Local,
    /// Sovereign custody.
    Sovereign,
    /// Escrowed custody.
    Escrow,
}

/// Handle to the key used for signing deletion proofs.
#[derive(Debug, Clone)]
pub enum SigningKeyHandle<'a> {
    /// Symmetric HMAC key slice.
    Hmac(&'a [u8]),
    /// Ed25519 signing key reference.
    Ed25519(&'a ed25519_dalek::SigningKey),
}

impl<'a> From<&'a [u8]> for SigningKeyHandle<'a> {
    fn from(key: &'a [u8]) -> Self {
        SigningKeyHandle::Hmac(key)
    }
}

impl<'a, const N: usize> From<&'a [u8; N]> for SigningKeyHandle<'a> {
    fn from(key: &'a [u8; N]) -> Self {
        SigningKeyHandle::Hmac(key.as_slice())
    }
}

impl<'a> From<&'a ed25519_dalek::SigningKey> for SigningKeyHandle<'a> {
    fn from(key: &'a ed25519_dalek::SigningKey) -> Self {
        SigningKeyHandle::Ed25519(key)
    }
}

/// Key parameter for verification (either HMAC-SHA256 byte slice or Ed25519 VerifyingKey).
#[derive(Debug, Clone, Copy)]
pub enum VerificationKey<'a> {
    /// Key for HMAC-SHA256 signature verification (version 1 and 2).
    Hmac(&'a [u8]),
    /// Key for Ed25519 signature verification (version 3).
    Ed25519(&'a ed25519_dalek::VerifyingKey),
}

impl<'a> From<&'a [u8]> for VerificationKey<'a> {
    fn from(key: &'a [u8]) -> Self {
        VerificationKey::Hmac(key)
    }
}

impl<'a, const N: usize> From<&'a [u8; N]> for VerificationKey<'a> {
    fn from(key: &'a [u8; N]) -> Self {
        VerificationKey::Hmac(key.as_slice())
    }
}

impl<'a> From<&'a Vec<u8>> for VerificationKey<'a> {
    fn from(key: &'a Vec<u8>) -> Self {
        VerificationKey::Hmac(key.as_slice())
    }
}

impl<'a> From<&'a ed25519_dalek::VerifyingKey> for VerificationKey<'a> {
    fn from(key: &'a ed25519_dalek::VerifyingKey) -> Self {
        VerificationKey::Ed25519(key)
    }
}

/// Beweis, dass ein bestimmter DeletionLayer physisch bereinigt wurde.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayerCleanupProof {
    pub(crate) layer: DeletionLayer,
    pub(crate) _private: (),
}

impl LayerCleanupProof {
    /// Erzeugt ein `LayerCleanupProof` ausschließlich für Tests.
    pub fn for_test(layer: DeletionLayer) -> Self {
        Self {
            layer,
            _private: (),
        }
    }

    /// Gibt den zugrundeliegenden DeletionLayer zurück.
    pub fn layer(&self) -> &DeletionLayer {
        &self.layer
    }
}

/// Layer-explizite Coverage-Deklaration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
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
    /// Full-text inverted index posting lists and tombstones purged.
    TextIndex,
}

/// Uncovered layer explanation for scope.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UncoveredLayer {
    /// Layer that was not covered in measurement.
    pub layer: DeletionLayer,
    /// Justification why layer was not covered for the given scope.
    pub reason: String,
}

mod scan_technique_serde {
    use contextra_ports::ScanTechnique;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    #[derive(Serialize, Deserialize)]
    enum ScanTechniqueSerde {
        Structural,
        DecryptedView,
        IndexIntrospection,
        NeedleScan,
        NotMeasured(String),
        FeatureDisabled,
    }

    pub fn serialize<S>(tech: &ScanTechnique, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let helper = match tech {
            ScanTechnique::Structural => ScanTechniqueSerde::Structural,
            ScanTechnique::DecryptedView => ScanTechniqueSerde::DecryptedView,
            ScanTechnique::IndexIntrospection => ScanTechniqueSerde::IndexIntrospection,
            ScanTechnique::NeedleScan => ScanTechniqueSerde::NeedleScan,
            ScanTechnique::NotMeasured(r) => ScanTechniqueSerde::NotMeasured(r.clone()),
            ScanTechnique::FeatureDisabled => ScanTechniqueSerde::FeatureDisabled,
        };
        helper.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<ScanTechnique, D::Error>
    where
        D: Deserializer<'de>,
    {
        let helper = ScanTechniqueSerde::deserialize(deserializer)?;
        Ok(match helper {
            ScanTechniqueSerde::Structural => ScanTechnique::Structural,
            ScanTechniqueSerde::DecryptedView => ScanTechnique::DecryptedView,
            ScanTechniqueSerde::IndexIntrospection => ScanTechnique::IndexIntrospection,
            ScanTechniqueSerde::NeedleScan => ScanTechnique::NeedleScan,
            ScanTechniqueSerde::NotMeasured(r) => ScanTechnique::NotMeasured(r),
            ScanTechniqueSerde::FeatureDisabled => ScanTechnique::FeatureDisabled,
        })
    }
}

/// Measurement details for a single witnessed layer in V5 proof.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LayerWitnessMeasurement {
    /// Layer that was measured.
    pub layer: DeletionLayer,
    /// Applied scan technique.
    #[serde(with = "scan_technique_serde")]
    pub technique: ScanTechnique,
    /// Number of units scanned during measurement.
    pub scanned: u64,
    /// Number of remaining live residual units (MUST be 0).
    pub remaining: u64,
    /// Number of passed positive control checks.
    pub control_matches: u64,
    /// BLAKE3 measurement digest.
    pub digest: [u8; 32],
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

/// Cryptographic proof of data deletion.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeletionProof {
    /// Version der Signatur-Payload-Konstruktion.
    #[serde(default = "default_signature_version")]
    pub signature_version: u8,
    /// Target scope of deletion.
    pub scope: DeletionScope,
    /// Blake3-Hash aller gelöschten Dokumentschlüssel (sortiert → deterministisch).
    pub deleted_keys_hash: [u8; 32],
    /// TxId nach der kein gelöschtes Datum mehr im System vorhanden ist.
    pub deleted_after_tx: TxId,
    /// Zeitstempel der Erstellung (Unix Timestamp in Sekunden).
    #[serde(default)]
    pub timestamp: u64,
    /// Signatur (32 Bytes für v1/v2 HMAC, 64 Bytes für v3 Ed25519).
    pub signature: Vec<u8>,
    /// List of physically sanitized storage layers.
    pub covered_layers: Vec<DeletionLayer>,
    /// Pflicht für DSGVO Art. 17-Compliance.
    pub excluded_scopes: Vec<ExcludedScope>,
    /// Attestierungen synchroner Nachbarschaftsreparaturen im HNSW-Graphen.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub graph_repair: Vec<GraphRepairAttestation>,
    /// Kryptographische Quittung H(hmac_prev || delete_event) der WAL-HMAC-Kette.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub wal_chain_receipt: Option<[u8; 32]>,
    /// Verweis auf die Position (Index) des zugehörigen Audit-Chain-Eintrags.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub audit_chain_position: Option<u64>,
    /// Custody level recorded during deletion proof generation.
    #[serde(default = "default_custody_level")]
    pub custody: CustodyLevel,
    /// Detailed measurements per layer for V5 proofs.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub measurements: Vec<LayerWitnessMeasurement>,
    /// Explicitly uncovered mandatory/optional layers with justification.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub uncovered_layers: Vec<UncoveredLayer>,
    /// Integritätswarnung für Legacy-Proofs (Version 1).
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub integrity_warning: Option<String>,
}

const fn default_custody_level() -> CustodyLevel {
    CustodyLevel::Local
}

const fn default_signature_version() -> u8 {
    1
}
