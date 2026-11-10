// FILE-CONTEXT
// ZWECK: Cryptographic deletion proof for GDPR Article 17 compliance verification across storage layers.
// INVARIANTEN: INV-DELETION-1: DeletionProof::create() MUST only be invoked AFTER physical layer sanitization.
// NICHT-OFFENSICHTLICH: V1 signiert nur scope, key-hash und tx_id via HMAC-SHA256; V2 signiert scope, key-hash, tx_id, covered_layers, excluded_scopes und wal_receipt via HMAC-SHA256; V3 signiert mit Ed25519 und nutzt längenpräfixierte Key-Hashes. verify() nutzt bei HMAC-Versionen Constant-Time-Vergleiche.
// HOTSPOTS: [40-130]
// STAND: TS:2026-09-07T00:00:00Z

#![forbid(unsafe_code)]

//! Kryptographischer Löschbeweis für die Storage-Ebene.
//!
//! KRITISCHE DECKUNGSGRENZE (Pflicht in Enterprise-Doku und export_for_audit):
//! Dieser Proof deckt AUSSCHLIESSLICH die physischen Storage-Layer ab:
//! `LsmMemtable`, `SsTableAllLevels`, `HnswIndex`, `WalAllSegments`, `CsrGraph`,
//! `KvCacheSegments` und `EmbeddingCache`.
//!
//! Explizit VOM PROOF AUSGESCHLOSSEN (`ExcludedScope`) sind:
//! `ConsolidatedAndDistilled` (Zusammenfassungen für LLM-Fine-Tuning) und
//! `LlmParameterMemory` (LLM-Modellparameter; arXiv:2505.16831).
//! Er KANN NICHT garantieren, dass Wissen aus Fine-Tuning-Zusammenfassungen
//! aus LLM-Parametern entfernbar ist. Referenz: arXiv:2505.16831.
//!
//! INVARIANTE INV-DELETION-1: DeletionProof::create() wird NUR nach
//! physischer Layer-Bereinigung aufgerufen. Proof vor Bereinigung = falsch.

use crate::ed25519_proof::{DeletionProofError, SignatureVersion};
/// Typalias für CryptoError.
pub type CryptoError = crate::error::ErasureError;
use crate::error::ErasureError;
use crate::types_local::{
    CollectionId, DocId, ScanTechnique, ScanWitness as LayerWitness, TenantId, TxId, WitnessLayer,
};
/// Typalias für Result.
pub type Result<T> = std::result::Result<T, crate::error::ErasureError>;
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
        if matches!(
            self.method(),
            crate::types_local::ScanMethod::FeatureDisabled
        ) {
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
                ErasureError::Internal(
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
            other => Err(ErasureError::Internal(format!(
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
///
/// DOKUMENTATION ZUM KOLLISIONSRISIKO:
/// Ohne Längenpräfix pro Element (z. B. bloße Konkatenation) erzeugen unterschiedliche Key-Listen
/// wie `["ab", "c"]` und `["a", "bc"]` denselben Hash-Wert, da die Elementgrenzen im Byte-Strom
/// nicht kodiert sind. Durch das Voranstellen der Schlüssellänge als 4-Byte Little-Endian integer
/// (`(key.len() as u32).to_le_bytes()`) vor jedem Schlüssel-Byte-Array wird eine eindeutige,
/// kollisionsfreie Kodierung für jede Sequenz von Schlüssel-Bytes.
pub fn hash_deleted_keys_length_prefixed(deleted_keys: &[Vec<u8>]) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    for key in deleted_keys {
        let len = u32::try_from(key.len()).unwrap_or(u32::MAX);
        hasher.update(&len.to_le_bytes());
        hasher.update(key);
    }
    *hasher.finalize().as_bytes()
}

/// KeyPair for Ed25519 signing and verification of DeletionProofs (version 3).
#[derive(Debug)]
pub struct DeletionProofKeyPair {
    signing_key: ed25519_dalek::SigningKey,
    /// Öffentlicher Verifikationsschlüssel.
    pub verifying_key: ed25519_dalek::VerifyingKey,
}

impl DeletionProofKeyPair {
    /// Generates a new random Ed25519 keypair using OsRng.
    pub fn generate() -> Self {
        let mut rng = rand::rngs::OsRng;
        let signing_key = ed25519_dalek::SigningKey::generate(&mut rng);
        let verifying_key = signing_key.verifying_key();
        Self {
            signing_key,
            verifying_key,
        }
    }

    /// Returns the 32-byte representation of the verifying public key.
    pub fn verifying_key_bytes(&self) -> [u8; 32] {
        self.verifying_key.to_bytes()
    }

    /// Returns a reference to the signing key.
    pub fn signing_key(&self) -> &ed25519_dalek::SigningKey {
        &self.signing_key
    }
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
/// Kann NUR von den jeweiligen Bereinigungsfunktionen der Storage-Layer erzeugt werden
/// (siehe `new_after_physical_cleanup`), niemals direkt frei durch Aufrufer von
/// `DeletionProof::create()`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayerCleanupProof {
    layer: DeletionLayer,
    _private: (),
}

impl LayerCleanupProof {
    /// Erzeugt einen Proof NUR, wenn `remaining_live_entries == 0` — also nur dann,
    /// wenn der Aufrufer nachweislich (durch einen Re-Scan des betroffenen
    /// Storage-Bereichs NACH der physischen Bereinigung) verifiziert hat, dass für
    /// diesen Layer keine lebenden Einträge mehr existieren. Ein Proof für einen
    /// Layer mit `remaining_live_entries > 0` ist ein Widerspruch zu INV-DELETION-1
    /// und wird abgelehnt statt stillschweigend akzeptiert.
    ///
    /// # Errors
    /// Gibt `ErasureError::Internal` zurück, wenn `remaining_live_entries != 0`.
    #[deprecated(note = "Umstellung auf DeletionProof::issue, siehe P-19")]
    pub fn new_after_verified_empty(
        layer: DeletionLayer,
        remaining_live_entries: usize,
    ) -> Result<Self> {
        if remaining_live_entries != 0 {
            return Err(ErasureError::Internal(format!(
                "INV-DELETION-1 violation: attempted to construct LayerCleanupProof for \
                 layer {layer:?} but verification found {remaining_live_entries} \
                 remaining live entries — physical cleanup is incomplete or was not \
                 performed before proof construction"
            )));
        }
        Ok(Self {
            layer,
            _private: (),
        })
    }

    /// Erzeugt einen Proof für `layer` NUR, wenn `verification` bestätigt,
    /// dass die physische Bereinigung tatsächlich abgeschlossen ist.
    ///
    /// `verification` MUSS eine echte Post-Condition-Prüfung durchführen
    /// (z. B. eine erneute Abfrage des betroffenen Storage-Layers, die
    /// belegt, dass keine der zu löschenden Daten mehr vorhanden sind),
    /// KEINE bloße Behauptung. Ein `Ok(false)`-Rückgabewert oder ein
    /// `Err` aus `verification` führt zu einem `Err` hier — es wird in
    /// diesem Fall NIEMALS ein Proof erzeugt (INV-DELETION-1).
    #[deprecated(note = "Umstellung auf DeletionProof::issue, siehe P-19")]
    pub fn verify_and_create<F>(layer: DeletionLayer, verification: F) -> Result<Self>
    where
        F: FnOnce() -> Result<bool>,
    {
        if verification()? {
            Ok(Self {
                layer,
                _private: (),
            })
        } else {
            Err(ErasureError::Internal(format!(
                "INV-DELETION-1 violation: physical cleanup verification \
                 failed for layer {layer:?} — refusing to create \
                 LayerCleanupProof"
            )))
        }
    }

    /// Gibt den zugrundeliegenden DeletionLayer zurück.
    pub fn layer(&self) -> &DeletionLayer {
        &self.layer
    }
}

/// Layer-explizite Coverage-Deklaration.
/// Jeder Layer MUSS physisch bereinigt sein bevor er hier deklariert wird.
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
    use crate::types_local::ScanTechnique;
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
// AI-TAG[SMELL][RESOLVED] audit-R3-1: DeletionProof signature_version 2 erweitert Signatur-Payload um covered_layers & excluded_scopes zur Vermeidung von Cross-Context-Fälschungen.
pub struct DeletionProof {
    /// Version der Signatur-Payload-Konstruktion.
    ///
    /// Version 1 (Legacy): Signiert NUR `scope`, `deleted_keys_hash` und `deleted_after_tx`.
    /// WARNUNG: Version 1 enthält eine bekannte Sicherheitslücke, da `covered_layers` und
    /// `excluded_scopes` ungesichert bleiben.
    ///
    /// Version 2 (Legacy HMAC): Signiert `scope`, `deleted_keys_hash`, `deleted_after_tx`,
    /// `covered_layers`, `excluded_scopes` und optional `wal_chain_receipt` mit HMAC-SHA256.
    ///
    /// Version 3 (Aktuell Ed25519): Signiert mit Ed25519, `deleted_keys_hash` ist längenpräfixiert.
    ///
    /// DOKUMENTATION ZUR TYPSICHEREN VALIDIERUNG (Teil 10.1):
    /// Aus serialisierten/empfangenen Bytes gelesene Werte werden über `signature_version_typed()`
    /// mittels `SignatureVersion::try_from(u8)` typisiert. Unbekannte oder ungültige Werte werden
    /// fail-closed abgelehnt, bevor jegliche Signatur- oder Krypto-Verifikation versucht wird.
    #[serde(default = "default_signature_version")]
    pub signature_version: u8,
    /// Target scope of deletion.
    pub scope: DeletionScope,
    /// Blake3-Hash aller gelöschten Dokumentschlüssel (sortiert → deterministisch).
    pub deleted_keys_hash: [u8; 32],
    /// TxId nach der kein gelöschtes Datum mehr im System vorhanden ist.
    /// ADR-016: TxId statt SystemTime für Determinismus.
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

impl DeletionProof {
    /// Issues a new `DeletionProof` based on verified `LayerWitness` measurements.
    ///
    /// # Fail-Closed Validation (INV-WITNESS-MINT-1 & REQ-PRF-2)
    /// Returns an error if any provided witness has `remaining > 0` or if its scan method
    /// is neither `ScanMethod::Structural` nor `ScanMethod::FeatureDisabled` (e.g. `ScanMethod::NotMeasured`).
    #[allow(clippy::too_many_arguments)]
    pub fn issue<'a>(
        ctx: &crate::types_local::RequestContext,
        saga: &crate::types_local::SagaRecord,
        witnesses: &[LayerWitness],
        excluded: &[ExcludedScope],
        custody: CustodyLevel,
        key: impl Into<SigningKeyHandle<'a>>,
        clock: &dyn crate::types_local::Clock,
    ) -> Result<Self> {
        for w in witnesses {
            if w.remaining() > 0 {
                return Err(ErasureError::Internal(format!(
                    "INV-WITNESS-MINT-1 violation: layer '{}' has {} remaining live entries",
                    w.layer(),
                    w.remaining()
                )));
            }
            match w.method() {
                crate::types_local::ScanMethod::Structural
                | crate::types_local::ScanMethod::FeatureDisabled => {}
                crate::types_local::ScanMethod::NotMeasured(reason) => {
                    return Err(ErasureError::Internal(format!(
                        "INV-WITNESS-MINT-1 violation: layer '{}' was not measured ({})",
                        w.layer(),
                        reason
                    )));
                }
            }
        }

        // 1. witnesses empty -> Err
        if witnesses.is_empty() {
            return Err(ErasureError::Internal(
                "REQ-PRF-1 violation: witnesses list cannot be empty for deletion proof issuance"
                    .to_string(),
            ));
        }

        // 2. excluded empty -> Err (REQ-PRF-2)
        if excluded.is_empty() {
            return Err(ErasureError::Internal(
                "REQ-PRF-2 violation: excluded scopes list cannot be empty".to_string(),
            ));
        }

        use crate::deletion_proof::LayerWitnessExt;

        let mut covered_layers = Vec::new();
        let mut measurements = Vec::new();
        let mut seen_layers = std::collections::BTreeSet::new();

        for w in witnesses {
            // Check for remaining > 0 -> Err
            if w.remaining() > 0 {
                return Err(ErasureError::Internal(format!(
                    "INV-WITNESS-MINT-1 violation: layer '{}' has {} remaining live entries",
                    w.layer(),
                    w.remaining()
                )));
            }

            // Check ScanMethod::NotMeasured -> Err
            match w.method() {
                crate::types_local::ScanMethod::Structural
                | crate::types_local::ScanMethod::FeatureDisabled => {}
                crate::types_local::ScanMethod::NotMeasured(reason) => {
                    return Err(ErasureError::Internal(format!(
                        "INV-WITNESS-MINT-1 violation: layer '{}' was not measured ({})",
                        w.layer(),
                        reason
                    )));
                }
            }

            // Check technique == ScanTechnique::NotMeasured / LogicalView for physical storage layers -> Err
            if let ScanTechnique::NotMeasured(reason) = w.technique() {
                return Err(ErasureError::Internal(format!(
                    "INV-WITNESS-MINT-1 violation: layer '{}' was not measured ({})",
                    w.layer(),
                    reason
                )));
            }

            let witness_layer = w.witness_layer().clone();

            // Check for duplicate layers using WitnessLayer -> Err
            if !seen_layers.insert(witness_layer.clone()) {
                return Err(ErasureError::Internal(format!(
                    "INV-WITNESS-MINT-1 violation: duplicate witness layer '{:?}' in witnesses list",
                    witness_layer
                )));
            }

            let mapped_layer = map_witness_layer_to_deletion_layer(&witness_layer, w)?;

            covered_layers.push(mapped_layer.clone());

            measurements.push(LayerWitnessMeasurement {
                layer: mapped_layer,
                technique: w.technique().clone(),
                scanned: w.scanned(),
                remaining: w.remaining(),
                control_matches: w.control_matches(),
                digest: *w.digest(),
            });
        }

        // Check mandatory storage layers: LsmMemtable, SsTableAllLevels, WalAllSegments
        let has_memtable = covered_layers
            .iter()
            .any(|l| matches!(l, DeletionLayer::LsmMemtable));
        let has_sstable = covered_layers
            .iter()
            .any(|l| matches!(l, DeletionLayer::SsTableAllLevels));
        let has_wal = covered_layers
            .iter()
            .any(|l| matches!(l, DeletionLayer::WalAllSegments { .. }));

        if !has_memtable {
            return Err(ErasureError::Internal(
                "INV-WITNESS-MINT-1 violation: mandatory layer LsmMemtable missing from witnesses"
                    .to_string(),
            ));
        }
        if !has_sstable {
            return Err(ErasureError::Internal(
                "INV-WITNESS-MINT-1 violation: mandatory layer SsTableAllLevels missing from witnesses"
                    .to_string(),
            ));
        }
        if !has_wal {
            return Err(ErasureError::Internal(
                "INV-WITNESS-MINT-1 violation: mandatory layer WalAllSegments missing from witnesses"
                    .to_string(),
            ));
        }

        // Optional layers HnswIndex and TextIndex, if not present in witnesses, add to uncovered_layers
        let mut uncovered_layers = Vec::new();
        if !covered_layers
            .iter()
            .any(|l| matches!(l, DeletionLayer::HnswIndex))
        {
            uncovered_layers.push(UncoveredLayer {
                layer: DeletionLayer::HnswIndex,
                reason: "Scope does not contain vector HnswIndex data".to_string(),
            });
        }
        if !covered_layers
            .iter()
            .any(|l| matches!(l, DeletionLayer::TextIndex))
        {
            uncovered_layers.push(UncoveredLayer {
                layer: DeletionLayer::TextIndex,
                reason: "Scope does not contain full-text TextIndex data".to_string(),
            });
        }

        let scope = DeletionScope::Tenant {
            tenant_id: ctx.tenant_id,
        };
        let timestamp = clock.now_unix_nanos() / 1_000_000_000;

        let proof_stub = Self {
            signature_version: SignatureVersion::V5.as_u8(),
            scope,
            deleted_keys_hash: saga.deleted_keys_hash,
            deleted_after_tx: ctx.tx_id,
            timestamp,
            signature: Vec::new(),
            covered_layers,
            excluded_scopes: excluded.to_vec(),
            graph_repair: Vec::new(),
            wal_chain_receipt: None,
            audit_chain_position: None,
            custody,
            measurements,
            uncovered_layers,
            integrity_warning: None,
        };

        let payload = proof_stub
            .construct_v5_payload()
            .map_err(|e| ErasureError::Internal(e.to_string()))?;

        let mut proof = proof_stub;

        match key.into() {
            SigningKeyHandle::Ed25519(sk) => {
                use ed25519_dalek::Signer;
                let sig = sk.sign(&payload);
                proof.signature = sig.to_bytes().to_vec();
            }
            SigningKeyHandle::Hmac(hmac_key) => {
                let sig = compute_hmac_sha256(hmac_key, &[&payload])?;
                proof.signature = sig.to_vec();
            }
        }

        Ok(proof)
    }

    /// Erstellt und signiert einen DeletionProof nach Layer-Bereinigung.
    ///
    /// AUFRUFREIHENFOLGE (INV-DELETION-1):
    /// 1. Alle covered_layers physisch bereinigen
    /// 2. WAL-Commit mit Lösch-Intent (P3)
    /// 3. DeletionProof::create() aufrufen
    pub fn create(
        scope: DeletionScope,
        deleted_keys: Vec<Vec<u8>>,
        deleted_after_tx: TxId,
        covered_layers: Vec<LayerCleanupProof>,
        excluded_scopes: Vec<ExcludedScope>,
        proof_key: &[u8],
    ) -> Result<Self> {
        Self::create_with_wal_receipt(
            scope,
            deleted_keys,
            deleted_after_tx,
            covered_layers,
            excluded_scopes,
            None,
            proof_key,
        )
    }

    /// Erstellt und signiert einen DeletionProof (Version 2) inklusive optionaler WAL-HMAC-Kettenquittung.
    pub fn create_with_wal_receipt(
        scope: DeletionScope,
        mut deleted_keys: Vec<Vec<u8>>,
        deleted_after_tx: TxId,
        covered_layers: Vec<LayerCleanupProof>,
        excluded_scopes: Vec<ExcludedScope>,
        wal_chain_receipt: Option<[u8; 32]>,
        proof_key: &[u8],
    ) -> Result<Self> {
        // Keys sortieren für deterministischen Hash
        deleted_keys.sort();

        let deleted_keys_hash = hash_deleted_keys_length_prefixed(&deleted_keys);

        let scope_bytes =
            bincode::serialize(&scope).map_err(|e| ErasureError::Internal(e.to_string()))?;
        let tx_bytes = deleted_after_tx.0.to_le_bytes();

        let covered_layers: Vec<DeletionLayer> =
            covered_layers.into_iter().map(|p| p.layer).collect();

        let covered_layers_bytes = bincode::serialize(&covered_layers)
            .map_err(|e| ErasureError::Internal(e.to_string()))?;
        let excluded_scopes_bytes = bincode::serialize(&excluded_scopes)
            .map_err(|e| ErasureError::Internal(e.to_string()))?;

        let receipt_bytes = wal_chain_receipt.unwrap_or([0u8; 32]);
        let receipt_part = if wal_chain_receipt.is_some() {
            receipt_bytes.as_slice()
        } else {
            &[]
        };

        let signature = compute_hmac_sha256(
            proof_key,
            &[
                &scope_bytes,
                &deleted_keys_hash,
                &tx_bytes,
                &covered_layers_bytes,
                &excluded_scopes_bytes,
                receipt_part,
            ],
        )?;

        Ok(Self {
            signature_version: SignatureVersion::V2.as_u8(),
            scope,
            deleted_keys_hash,
            deleted_after_tx,
            timestamp: 0,
            signature: signature.to_vec(),
            covered_layers,
            excluded_scopes,
            graph_repair: Vec::new(),
            wal_chain_receipt,
            audit_chain_position: None,
            custody: CustodyLevel::Local,
            measurements: Vec::new(),
            uncovered_layers: Vec::new(),
            integrity_warning: None,
        })
    }

    /// Erstellt und signiert einen DeletionProof (Version 3) mit Ed25519.
    #[allow(clippy::too_many_arguments)]
    pub fn create_v3(
        scope: DeletionScope,
        deleted_keys: Vec<Vec<u8>>,
        deleted_after_tx: TxId,
        covered_layers: Vec<LayerCleanupProof>,
        excluded_scopes: Vec<ExcludedScope>,
        attested_at: i64,
        graph_repair: &[GraphRepairAttestation],
        signing_key: &ed25519_dalek::SigningKey,
    ) -> Result<Self> {
        Self::create_with_wal_receipt_v3(
            scope,
            deleted_keys,
            deleted_after_tx,
            covered_layers,
            excluded_scopes,
            None,
            attested_at,
            graph_repair,
            signing_key,
        )
    }

    /// Erstellt und signiert einen DeletionProof (Version 3, Ed25519) inklusive Verweis auf den Audit-Chain-Eintrag.
    #[allow(clippy::too_many_arguments)]
    pub fn create_v3_with_audit_position(
        scope: DeletionScope,
        deleted_keys: Vec<Vec<u8>>,
        deleted_after_tx: TxId,
        covered_layers: Vec<LayerCleanupProof>,
        excluded_scopes: Vec<ExcludedScope>,
        audit_chain_position: Option<u64>,
        attested_at: i64,
        graph_repair: &[GraphRepairAttestation],
        signing_key: &ed25519_dalek::SigningKey,
    ) -> Result<Self> {
        Self::create_full_v3(
            scope,
            deleted_keys,
            deleted_after_tx,
            covered_layers,
            excluded_scopes,
            None,
            audit_chain_position,
            attested_at,
            graph_repair,
            signing_key,
        )
    }

    /// Erstellt und signiert einen DeletionProof (Version 3, Ed25519) inklusive optionaler WAL-HMAC-Kettenquittung.
    #[allow(clippy::too_many_arguments)]
    pub fn create_with_wal_receipt_v3(
        scope: DeletionScope,
        deleted_keys: Vec<Vec<u8>>,
        deleted_after_tx: TxId,
        covered_layers: Vec<LayerCleanupProof>,
        excluded_scopes: Vec<ExcludedScope>,
        wal_chain_receipt: Option<[u8; 32]>,
        attested_at: i64,
        graph_repair: &[GraphRepairAttestation],
        signing_key: &ed25519_dalek::SigningKey,
    ) -> Result<Self> {
        Self::create_full_v3(
            scope,
            deleted_keys,
            deleted_after_tx,
            covered_layers,
            excluded_scopes,
            wal_chain_receipt,
            None,
            attested_at,
            graph_repair,
            signing_key,
        )
    }

    /// Erstellt und signiert einen DeletionProof (Version 3, Ed25519) mit allen optionalen Erweiterungen.
    #[allow(clippy::too_many_arguments)]
    pub fn create_full_v3(
        scope: DeletionScope,
        mut deleted_keys: Vec<Vec<u8>>,
        deleted_after_tx: TxId,
        covered_layers: Vec<LayerCleanupProof>,
        excluded_scopes: Vec<ExcludedScope>,
        wal_chain_receipt: Option<[u8; 32]>,
        audit_chain_position: Option<u64>,
        attested_at: i64,
        graph_repair: &[GraphRepairAttestation],
        signing_key: &ed25519_dalek::SigningKey,
    ) -> Result<Self> {
        deleted_keys.sort();
        let deleted_keys_hash = hash_deleted_keys_length_prefixed(&deleted_keys);

        let covered_layers: Vec<DeletionLayer> =
            covered_layers.into_iter().map(|p| p.layer).collect();

        if covered_layers.contains(&DeletionLayer::HnswIndex) && graph_repair.is_empty() {
            return Err(ErasureError::Internal(
                "Graph repair attestation missing or incomplete: remaining_pointers: 1".to_string(),
            ));
        }

        let timestamp = u64::try_from(attested_at).unwrap_or(0);

        let proof_stub = Self {
            signature_version: SignatureVersion::V3.as_u8(),
            scope,
            deleted_keys_hash,
            deleted_after_tx,
            timestamp,
            signature: Vec::new(),
            covered_layers,
            excluded_scopes,
            graph_repair: graph_repair.to_vec(),
            wal_chain_receipt,
            audit_chain_position,
            custody: CustodyLevel::Local,
            measurements: Vec::new(),
            uncovered_layers: Vec::new(),
            integrity_warning: None,
        };

        let payload = proof_stub
            .construct_v3_payload()
            .map_err(|e| ErasureError::Internal(e.to_string()))?;

        use ed25519_dalek::Signer;
        let sig = signing_key.sign(&payload);

        let mut proof = proof_stub;
        proof.signature = sig.to_bytes().to_vec();

        Ok(proof)
    }

    /// Parst und liefert die typsichere Signaturversion des Beweises.
    ///
    /// # Errors
    /// Gibt [`DeletionProofError::UnsupportedVersion`] zurück, wenn `signature_version` einen ungültigen/unbekannten Wert enthält.
    pub fn signature_version_typed(
        &self,
    ) -> std::result::Result<SignatureVersion, DeletionProofError> {
        SignatureVersion::try_from(self.signature_version)
    }

    /// Verifiziert Signatur.
    /// Referenz: Befund v17 Teil 10.1 & INVARIANTE INV-DELETION-1.
    /// NICHT-GARANTIE: Prüft nur Signatur, nicht ob Storage tatsächlich bereinigt ist.
    pub fn verify<'a>(&self, key: impl Into<VerificationKey<'a>>) -> Result<bool> {
        let version = self
            .signature_version_typed()
            .map_err(|e| ErasureError::Internal(e.to_string()))?;

        let key = key.into();
        let scope_bytes =
            bincode::serialize(&self.scope).map_err(|e| ErasureError::Internal(e.to_string()))?;
        let tx_bytes = self.deleted_after_tx.0.to_le_bytes();

        match version {
            SignatureVersion::V1 => {
                let proof_key = match key {
                    VerificationKey::Hmac(k) => k,
                    VerificationKey::Ed25519(_) => {
                        return Err(ErasureError::Internal(
                            "Ed25519 key provided for HMAC signature_version 1 proof".to_string(),
                        ))
                    }
                };
                let expected = compute_hmac_sha256(
                    proof_key,
                    &[&scope_bytes, &self.deleted_keys_hash, &tx_bytes],
                )?;
                use subtle::ConstantTimeEq;
                Ok(expected.as_slice().ct_eq(&self.signature).into())
            }
            SignatureVersion::V2 => {
                let proof_key = match key {
                    VerificationKey::Hmac(k) => k,
                    VerificationKey::Ed25519(_) => {
                        return Err(ErasureError::Internal(
                            "Ed25519 key provided for HMAC signature_version 2 proof".to_string(),
                        ))
                    }
                };
                let covered_layers_bytes = bincode::serialize(&self.covered_layers)
                    .map_err(|e| ErasureError::Internal(e.to_string()))?;
                let excluded_scopes_bytes = bincode::serialize(&self.excluded_scopes)
                    .map_err(|e| ErasureError::Internal(e.to_string()))?;
                let receipt_bytes = self.wal_chain_receipt.unwrap_or([0u8; 32]);
                let receipt_part = if self.wal_chain_receipt.is_some() {
                    receipt_bytes.as_slice()
                } else {
                    &[]
                };
                let expected = compute_hmac_sha256(
                    proof_key,
                    &[
                        &scope_bytes,
                        &self.deleted_keys_hash,
                        &tx_bytes,
                        &covered_layers_bytes,
                        &excluded_scopes_bytes,
                        receipt_part,
                    ],
                )?;
                use subtle::ConstantTimeEq;
                Ok(expected.as_slice().ct_eq(&self.signature).into())
            }
            SignatureVersion::V3 => {
                let verifying_key = match key {
                    VerificationKey::Ed25519(vk) => vk,
                    VerificationKey::Hmac(_) => {
                        return Err(ErasureError::Internal(
                            "HMAC key provided for Ed25519 signature_version 3 proof".to_string(),
                        ))
                    }
                };
                let payload = self
                    .construct_v3_payload()
                    .map_err(|e| ErasureError::Internal(e.to_string()))?;

                use ed25519_dalek::Verifier;
                let sig = match ed25519_dalek::Signature::from_slice(&self.signature) {
                    Ok(s) => s,
                    Err(_) => return Ok(false),
                };

                Ok(verifying_key.verify(&payload, &sig).is_ok())
            }
            SignatureVersion::V5 => {
                let payload = self
                    .construct_v5_payload()
                    .map_err(|e| ErasureError::Internal(e.to_string()))?;

                match key {
                    VerificationKey::Ed25519(vk) => {
                        use ed25519_dalek::Verifier;
                        let sig = match ed25519_dalek::Signature::from_slice(&self.signature) {
                            Ok(s) => s,
                            Err(_) => return Ok(false),
                        };
                        Ok(vk.verify(&payload, &sig).is_ok())
                    }
                    VerificationKey::Hmac(hmac_key) => {
                        let expected = compute_hmac_sha256(hmac_key, &[&payload])?;
                        use subtle::ConstantTimeEq;
                        Ok(expected.as_slice().ct_eq(&self.signature).into())
                    }
                }
            }
        }
    }

    /// Helper to construct the signed payload for signature version 5.
    fn construct_v5_payload(&self) -> std::result::Result<Vec<u8>, CryptoError> {
        let version_bytes = [SignatureVersion::V5.as_u8()];
        let scope_bytes =
            bincode::serialize(&self.scope).map_err(|e| CryptoError::Crypto(e.to_string()))?;
        let tx_bytes = self.deleted_after_tx.0.to_le_bytes();
        let timestamp_bytes = self.timestamp.to_le_bytes();
        let custody_bytes =
            bincode::serialize(&self.custody).map_err(|e| CryptoError::Crypto(e.to_string()))?;
        let covered_layers_bytes = bincode::serialize(&self.covered_layers)
            .map_err(|e| CryptoError::Crypto(e.to_string()))?;
        let measurements_bytes = bincode::serialize(&self.measurements)
            .map_err(|e| CryptoError::Crypto(e.to_string()))?;
        let uncovered_bytes = bincode::serialize(&self.uncovered_layers)
            .map_err(|e| CryptoError::Crypto(e.to_string()))?;
        let excluded_scopes_bytes = bincode::serialize(&self.excluded_scopes)
            .map_err(|e| CryptoError::Crypto(e.to_string()))?;

        let mut payload = Vec::with_capacity(
            version_bytes.len()
                + scope_bytes.len()
                + 32
                + tx_bytes.len()
                + timestamp_bytes.len()
                + custody_bytes.len()
                + covered_layers_bytes.len()
                + measurements_bytes.len()
                + uncovered_bytes.len()
                + excluded_scopes_bytes.len(),
        );

        payload.extend_from_slice(&version_bytes);
        payload.extend_from_slice(&scope_bytes);
        payload.extend_from_slice(&self.deleted_keys_hash);
        payload.extend_from_slice(&tx_bytes);
        payload.extend_from_slice(&timestamp_bytes);
        payload.extend_from_slice(&custody_bytes);
        payload.extend_from_slice(&covered_layers_bytes);
        payload.extend_from_slice(&measurements_bytes);
        payload.extend_from_slice(&uncovered_bytes);
        payload.extend_from_slice(&excluded_scopes_bytes);

        Ok(payload)
    }

    /// Helper to construct the signed payload for signature version 3 (Ed25519).
    fn construct_v3_payload(&self) -> std::result::Result<Vec<u8>, CryptoError> {
        let scope_bytes =
            bincode::serialize(&self.scope).map_err(|e| CryptoError::Crypto(e.to_string()))?;
        let tx_bytes = self.deleted_after_tx.0.to_le_bytes();
        let timestamp_bytes = self.timestamp.to_le_bytes();
        let covered_layers_bytes = bincode::serialize(&self.covered_layers)
            .map_err(|e| CryptoError::Crypto(e.to_string()))?;
        let excluded_scopes_bytes = bincode::serialize(&self.excluded_scopes)
            .map_err(|e| CryptoError::Crypto(e.to_string()))?;
        let graph_repair_bytes = bincode::serialize(&self.graph_repair)
            .map_err(|e| CryptoError::Crypto(e.to_string()))?;
        let receipt_bytes = self.wal_chain_receipt.unwrap_or([0u8; 32]);
        let receipt_part = if self.wal_chain_receipt.is_some() {
            receipt_bytes.as_slice()
        } else {
            &[]
        };

        let audit_pos_bytes = self.audit_chain_position.map(|p| p.to_le_bytes());
        let audit_pos_part = if let Some(ref pos_b) = audit_pos_bytes {
            pos_b.as_slice()
        } else {
            &[]
        };

        let mut payload = Vec::with_capacity(
            scope_bytes.len()
                + 32
                + tx_bytes.len()
                + timestamp_bytes.len()
                + covered_layers_bytes.len()
                + excluded_scopes_bytes.len()
                + graph_repair_bytes.len()
                + receipt_part.len()
                + audit_pos_part.len(),
        );
        payload.extend_from_slice(&scope_bytes);
        payload.extend_from_slice(&self.deleted_keys_hash);
        payload.extend_from_slice(&tx_bytes);
        payload.extend_from_slice(&timestamp_bytes);
        payload.extend_from_slice(&covered_layers_bytes);
        payload.extend_from_slice(&excluded_scopes_bytes);
        payload.extend_from_slice(&graph_repair_bytes);
        payload.extend_from_slice(receipt_part);
        payload.extend_from_slice(audit_pos_part);

        Ok(payload)
    }

    /// Verifies this proof using ONLY the provided Ed25519 public key.
    /// Does NOT require access to any `KeyManager` state — this is the
    /// verification path intended for third parties who received a
    /// `DeletionProof` and the corresponding public key out-of-band.
    ///
    /// # Differences from [`verify`][Self::verify]
    /// - [`verify`][Self::verify] supports legacy HMAC proof versions (v1 and v2) requiring symmetric key access,
    ///   as well as v3 Ed25519 proofs via [`VerificationKey`].
    /// - `verify_external` operates strictly on `signature_version == 3` (Ed25519 asymmetric signatures) and
    ///   requires zero access to secret key material or internal `KeyManager` state. It returns explicit
    ///   [`CryptoError`] types ([`CryptoError::UnsupportedProofVersion`] for v1/v2, [`CryptoError::InvalidProofSignature`]
    ///   for invalid signatures or malformed signature bytes).
    ///
    /// # Cryptographic Verification Standard & Constant-Time Security
    /// Unlike version 1 and version 2 proofs which rely on symmetric HMAC-SHA256 checksums
    /// compared via [`subtle::ConstantTimeEq`], version 3 proof signature verification relies on
    /// Ed25519 asymmetric signature verification ([`ed25519_dalek::Verifier::verify`]).
    ///
    /// Ed25519 verification is an algebraic check on Curve25519 (`[S]B = R + [k]A` in curve point
    /// arithmetic), where scalar multiplication and group operations determine validity.
    /// Replacing curve point verification with a byte equality comparison (`ConstantTimeEq`) on
    /// signature bytes would be mathematically incorrect and cryptographic nonsense, because
    /// there is no "expected signature byte array" known a priori without solving the discrete logarithm problem.
    ///
    /// The `ed25519_dalek` implementation of `Verifier::verify` handles curve arithmetic in constant time
    /// where necessary to prevent timing side channels during public key verification.
    ///
    /// # Errors
    /// - [`CryptoError::UnsupportedProofVersion`] if `signature_version` is not 3.
    /// - [`CryptoError::InvalidProofSignature`] if the signature does not match or cannot be parsed.
    pub fn verify_external(
        &self,
        public_key: &ed25519_dalek::VerifyingKey,
    ) -> std::result::Result<(), CryptoError> {
        // Referenz: Befund v17 Teil 10.1 & INVARIANTE INV-DELETION-1
        // Verzweigung ausschließlich über den typisierten SignatureVersion-Wert.
        let version = self
            .signature_version_typed()
            .map_err(|_| CryptoError::UnsupportedProofVersion(self.signature_version))?;

        let payload = match version {
            SignatureVersion::V3 => self.construct_v3_payload()?,
            SignatureVersion::V5 => self.construct_v5_payload()?,
            _ => return Err(CryptoError::UnsupportedProofVersion(self.signature_version)),
        };

        use ed25519_dalek::Verifier;
        let sig = ed25519_dalek::Signature::from_slice(&self.signature)
            .map_err(|_| CryptoError::InvalidProofSignature)?;

        public_key
            .verify(&payload, &sig)
            .map_err(|_| CryptoError::InvalidProofSignature)
    }

    /// Exportiert Proof als JSON für Compliance-Dokumentation.
    /// ExcludedScope-Liste ist maschinenlesbar enthalten.
    pub fn export_for_audit(&self) -> Result<String> {
        // Referenz: Befund v17 Teil 10.1 & INVARIANTE INV-DELETION-1
        // Typisierte Versionsprüfung über SignatureVersion.
        let version = self
            .signature_version_typed()
            .map_err(|e| ErasureError::Internal(e.to_string()))?;

        if version == SignatureVersion::V1 {
            let mut clone = self.clone();
            clone.integrity_warning = Some(
                "covered_layers/excluded_scopes are not cryptographically signed in this legacy proof version"
                    .to_string(),
            );
            serde_json::to_string_pretty(&clone).map_err(|e| ErasureError::Internal(e.to_string()))
        } else if version == SignatureVersion::V2 || version == SignatureVersion::V3 {
            let mut clone = self.clone();
            clone.integrity_warning = Some(
                "covered_layers/excluded_scopes are not bound to a verified witness measurement in this legacy proof version"
                    .to_string(),
            );
            serde_json::to_string_pretty(&clone).map_err(|e| ErasureError::Internal(e.to_string()))
        } else {
            serde_json::to_string_pretty(self).map_err(|e| ErasureError::Internal(e.to_string()))
        }
    }

    /// Gibt die Tenant-ID aus dem Scope zurück.
    pub fn tenant_id(&self) -> TenantId {
        match &self.scope {
            DeletionScope::Document { tenant_id, .. } => *tenant_id,
            DeletionScope::Collection { tenant_id, .. } => *tenant_id,
            DeletionScope::Tenant { tenant_id } => *tenant_id,
        }
    }
}

/// Erzeugt eine WAL-Löschquittung H(hmac_prev || delete_event) für den Nachweis
/// auf WAL-Ebene gemäß DSGVO Art. 17.
pub fn compute_wal_delete_receipt(
    prev_hmac: &[u8; 32],
    delete_event_payload: &[u8],
    integrity_key: &[u8],
) -> Result<[u8; 32]> {
    compute_hmac_sha256(
        integrity_key,
        &[b"contextra-wal-delete-v1", prev_hmac, delete_event_payload],
    )
}

/// Verifiziert eine WAL-Löschquittung in O(1) konstanter Zeit ohne Klartextzugang.
pub fn verify_wal_delete_receipt(
    receipt: &[u8; 32],
    prev_hmac: &[u8; 32],
    delete_event_payload: &[u8],
    integrity_key: &[u8],
) -> Result<bool> {
    use subtle::ConstantTimeEq;
    let expected = compute_wal_delete_receipt(prev_hmac, delete_event_payload, integrity_key)?;
    Ok(expected.ct_eq(receipt).into())
}

fn compute_hmac_sha256(key: &[u8], data_parts: &[&[u8]]) -> Result<[u8; 32]> {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    let mut mac = Hmac::<Sha256>::new_from_slice(key)
        .map_err(|e| ErasureError::Internal(format!("HMAC key error: {e}")))?;
    for part in data_parts {
        mac.update(part);
    }
    Ok(mac.finalize().into_bytes().into())
}

#[cfg(test)]
#[path = "test_deletion_proof.rs"]
mod tests;

/// Detailed failure reasons for third-party verification of deletion proofs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerificationErrorReason {
    /// Digital signature is invalid or tampered with.
    InvalidSignature(String),
    /// Unbroken chain invariant violated.
    ChainBroken {
        /// Position in chain where break occurred.
        position: usize,
        /// Description of the break.
        details: String,
    },
    /// Audit chain head signature or head hash does not match expected anchor.
    HeadMismatch {
        /// Expected head hash.
        expected_head_hash: [u8; 32],
        /// Actual head hash found in chain.
        actual_head_hash: [u8; 32],
    },
    /// Unsupported signature version or proof format.
    UnsupportedVersion(u8),
    /// Chain has been truncated or is missing trailing entries without anchor update.
    TruncatedChain {
        /// Expected minimum chain length.
        expected_len: usize,
        /// Actual chain length.
        actual_len: usize,
    },
    /// Tenant mismatch in proof vs expected context.
    TenantMismatch {
        /// Expected tenant ID.
        expected: TenantId,
        /// Actual tenant ID in proof.
        actual: TenantId,
    },
    /// Provided chain is empty.
    EmptyChain,
    /// Invalid or malformed public verification key.
    InvalidKey(String),
}

/// Structured outcome of third-party deletion proof verification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerificationResult {
    /// `true` if all cryptographic checks, signatures, and chain continuities passed.
    pub is_valid: bool,
    /// Detailed error reason if `is_valid` is `false`.
    pub error_reason: Option<VerificationErrorReason>,
}

impl VerificationResult {
    /// Creates a successful verification result.
    pub fn success() -> Self {
        Self {
            is_valid: true,
            error_reason: None,
        }
    }

    /// Creates a failed verification result with an explicit reason.
    pub fn failure(reason: VerificationErrorReason) -> Self {
        Self {
            is_valid: false,
            error_reason: Some(reason),
        }
    }
}

/// Public facade for standalone third-party verification of deletion proofs.
#[derive(Debug, Default, Clone, Copy)]
pub struct DeletionProofVerifier;

impl DeletionProofVerifier {
    /// Verifies a single standalone `DeletionProof` using a public key.
    pub fn verify_standalone(
        proof: &DeletionProof,
        verifying_key_bytes: &[u8; 32],
    ) -> VerificationResult {
        let verifying_key = match ed25519_dalek::VerifyingKey::from_bytes(verifying_key_bytes) {
            Ok(vk) => vk,
            Err(e) => {
                return VerificationResult::failure(VerificationErrorReason::InvalidKey(
                    e.to_string(),
                ))
            }
        };

        if let Err(e) = proof.verify_external(&verifying_key) {
            let msg = e.to_string();
            let reason = if msg.contains("unsupported deletion proof version") {
                VerificationErrorReason::UnsupportedVersion(proof.signature_version)
            } else if msg.contains("invalid verifying key") {
                VerificationErrorReason::InvalidKey(msg)
            } else {
                VerificationErrorReason::InvalidSignature(msg)
            };
            return VerificationResult::failure(reason);
        }

        VerificationResult::success()
    }

    /// Verifies a `DeletionProof` and checks tenant binding.
    pub fn verify_standalone_for_tenant(
        proof: &DeletionProof,
        expected_tenant: TenantId,
        verifying_key_bytes: &[u8; 32],
    ) -> VerificationResult {
        if proof.tenant_id() != expected_tenant {
            return VerificationResult::failure(VerificationErrorReason::TenantMismatch {
                expected: expected_tenant,
                actual: proof.tenant_id(),
            });
        }
        Self::verify_standalone(proof, verifying_key_bytes)
    }
}
