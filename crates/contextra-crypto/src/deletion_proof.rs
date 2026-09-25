// FILE-CONTEXT
// ZWECK: Cryptographic deletion proof for GDPR Article 17 compliance verification across storage layers.
// INVARIANTEN: INV-DELETION-1: DeletionProof::create() MUST only be invoked AFTER physical layer sanitization.
// NICHT-OFFENSICHTLICH: HMAC-SHA256 signature covers scope, sorted key hash, and tx_id. verify() uses constant-time comparison.
// HOTSPOTS: [40-130]
// STAND: TS:2026-09-07T00:00:00Z

#![forbid(unsafe_code)]

//! Kryptographischer Löschbeweis für die Storage-Ebene.
//!
//! KRITISCHE DECKUNGSGRENZE (Pflicht in Enterprise-Doku und export_for_audit):
//! Dieser Proof deckt AUSSCHLIESSLICH: LSM, WAL, HNSW, CSR, KV-Cache.
//! Er KANN NICHT garantieren, dass Wissen aus Fine-Tuning-Zusammenfassungen
//! aus LLM-Parametern entfernbar ist. Referenz: arXiv:2505.16831.
//!
//! INVARIANTE INV-DELETION-1: DeletionProof::create() wird NUR nach
//! physischer Layer-Bereinigung aufgerufen. Proof vor Bereinigung = falsch.

#[cfg(test)]
use contextra_crypto::error::CryptoError;
#[cfg(not(test))]
use crate::error::CryptoError;
use contextra_types::{CollectionId, ContextraError, DocId, Result, TenantId, TxId};
use serde::{Deserialize, Serialize};

/// Berechnet den Blake3-Hash einer deterministisch sortierten Liste gelöschter Schlüssel mit Längenpräfix.
///
/// DOKUMENTATION ZUM KOLLISIONSRISIKO:
/// Ohne Längenpräfix pro Element (z. B. bloße Konkatenation) erzeugen unterschiedliche Key-Listen
/// wie `["ab", "c"]` und `["a", "bc"]` denselben Hash-Wert, da die Elementgrenzen im Byte-Strom
/// nicht kodiert sind. Durch das Voranstellen der Schlüssellänge als 8-Byte Big-Endian integer
/// (`(key.len() as u64).to_be_bytes()`) vor jedem Schlüssel-Byte-Array wird eine eindeutige,
/// kollisionsfreie Kodierung für jede Sequenz von Schlüssel-Bytes garantiert.
pub fn hash_deleted_keys_length_prefixed(deleted_keys: &[Vec<u8>]) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    for key in deleted_keys {
        hasher.update(&(key.len() as u64).to_be_bytes());
        hasher.update(key);
    }
    *hasher.finalize().as_bytes()
}

/// KeyPair for Ed25519 signing and verification of DeletionProofs (version 3).
#[derive(Debug)]
pub struct DeletionProofKeyPair {
    signing_key: ed25519_dalek::SigningKey,
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
    /// Gibt `ContextraError::Internal` zurück, wenn `remaining_live_entries != 0`.
    pub fn new_after_verified_empty(
        layer: DeletionLayer,
        remaining_live_entries: usize,
    ) -> Result<Self> {
        if remaining_live_entries != 0 {
            return Err(ContextraError::Internal(format!(
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

    /// Fabrikfunktion — MUSS unmittelbar nach erfolgreicher, verifizierter physischer
    /// Bereinigung (LSM-Compaction, WAL-Truncation, HNSW-Purge, ...) aufgerufen werden,
    /// um einen Proof für den jeweiligen Layer zu erzeugen.
    #[deprecated(
        note = "use new_after_verified_empty, which requires proof of an actual empty post-cleanup scan"
    )]
    #[allow(dead_code)]
    pub(crate) fn new_after_physical_cleanup(layer: DeletionLayer) -> Self {
        Self {
            layer,
            _private: (),
        }
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
            Err(ContextraError::Internal(format!(
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
    /// Kryptographische Quittung H(hmac_prev || delete_event) der WAL-HMAC-Kette.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub wal_chain_receipt: Option<[u8; 32]>,
    /// Integritätswarnung für Legacy-Proofs (Version 1).
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub integrity_warning: Option<String>,
}

const fn default_signature_version() -> u8 {
    1
}

impl DeletionProof {
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
            bincode::serialize(&scope).map_err(|e| ContextraError::Internal(e.to_string()))?;
        let tx_bytes = deleted_after_tx.0.to_le_bytes();

        let covered_layers: Vec<DeletionLayer> =
            covered_layers.into_iter().map(|p| p.layer).collect();

        let covered_layers_bytes = bincode::serialize(&covered_layers)
            .map_err(|e| ContextraError::Internal(e.to_string()))?;
        let excluded_scopes_bytes = bincode::serialize(&excluded_scopes)
            .map_err(|e| ContextraError::Internal(e.to_string()))?;

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
            signature_version: 2,
            scope,
            deleted_keys_hash,
            deleted_after_tx,
            timestamp: 0,
            signature: signature.to_vec(),
            covered_layers,
            excluded_scopes,
            wal_chain_receipt,
            integrity_warning: None,
        })
    }

    /// Erstellt und signiert einen DeletionProof (Version 3) mit Ed25519.
    pub fn create_v3(
        scope: DeletionScope,
        deleted_keys: Vec<Vec<u8>>,
        deleted_after_tx: TxId,
        covered_layers: Vec<LayerCleanupProof>,
        excluded_scopes: Vec<ExcludedScope>,
        signing_key: &ed25519_dalek::SigningKey,
    ) -> Result<Self> {
        Self::create_with_wal_receipt_v3(
            scope,
            deleted_keys,
            deleted_after_tx,
            covered_layers,
            excluded_scopes,
            None,
            signing_key,
        )
    }

    /// Erstellt und signiert einen DeletionProof (Version 3, Ed25519) inklusive optionaler WAL-HMAC-Kettenquittung.
    pub fn create_with_wal_receipt_v3(
        scope: DeletionScope,
        mut deleted_keys: Vec<Vec<u8>>,
        deleted_after_tx: TxId,
        covered_layers: Vec<LayerCleanupProof>,
        excluded_scopes: Vec<ExcludedScope>,
        wal_chain_receipt: Option<[u8; 32]>,
        signing_key: &ed25519_dalek::SigningKey,
    ) -> Result<Self> {
        deleted_keys.sort();
        let deleted_keys_hash = hash_deleted_keys_length_prefixed(&deleted_keys);

        let scope_bytes =
            bincode::serialize(&scope).map_err(|e| ContextraError::Internal(e.to_string()))?;
        let tx_bytes = deleted_after_tx.0.to_le_bytes();

        let covered_layers: Vec<DeletionLayer> =
            covered_layers.into_iter().map(|p| p.layer).collect();

        let covered_layers_bytes = bincode::serialize(&covered_layers)
            .map_err(|e| ContextraError::Internal(e.to_string()))?;
        let excluded_scopes_bytes = bincode::serialize(&excluded_scopes)
            .map_err(|e| ContextraError::Internal(e.to_string()))?;

        let receipt_bytes = wal_chain_receipt.unwrap_or([0u8; 32]);
        let receipt_part = if wal_chain_receipt.is_some() {
            receipt_bytes.as_slice()
        } else {
            &[]
        };

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let mut payload = Vec::new();
        payload.extend_from_slice(&scope_bytes);
        payload.extend_from_slice(&deleted_keys_hash);
        payload.extend_from_slice(&tx_bytes);
        payload.extend_from_slice(&timestamp.to_le_bytes());
        payload.extend_from_slice(&covered_layers_bytes);
        payload.extend_from_slice(&excluded_scopes_bytes);
        payload.extend_from_slice(receipt_part);

        use ed25519_dalek::Signer;
        let sig = signing_key.sign(&payload);

        Ok(Self {
            signature_version: 3,
            scope,
            deleted_keys_hash,
            deleted_after_tx,
            timestamp,
            signature: sig.to_bytes().to_vec(),
            covered_layers,
            excluded_scopes,
            wal_chain_receipt,
            integrity_warning: None,
        })
    }

    /// Verifiziert Signatur.
    /// NICHT-GARANTIE: Prüft nur Signatur, nicht ob Storage tatsächlich bereinigt ist.
    pub fn verify<'a>(&self, key: impl Into<VerificationKey<'a>>) -> Result<bool> {
        let key = key.into();
        let scope_bytes =
            bincode::serialize(&self.scope).map_err(|e| ContextraError::Internal(e.to_string()))?;
        let tx_bytes = self.deleted_after_tx.0.to_le_bytes();

        match self.signature_version {
            1 => {
                let proof_key = match key {
                    VerificationKey::Hmac(k) => k,
                    VerificationKey::Ed25519(_) => {
                        return Err(ContextraError::Internal(
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
            2 => {
                let proof_key = match key {
                    VerificationKey::Hmac(k) => k,
                    VerificationKey::Ed25519(_) => {
                        return Err(ContextraError::Internal(
                            "Ed25519 key provided for HMAC signature_version 2 proof".to_string(),
                        ))
                    }
                };
                let covered_layers_bytes = bincode::serialize(&self.covered_layers)
                    .map_err(|e| ContextraError::Internal(e.to_string()))?;
                let excluded_scopes_bytes = bincode::serialize(&self.excluded_scopes)
                    .map_err(|e| ContextraError::Internal(e.to_string()))?;
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
            3 => {
                let verifying_key = match key {
                    VerificationKey::Ed25519(vk) => vk,
                    VerificationKey::Hmac(_) => {
                        return Err(ContextraError::Internal(
                            "HMAC key provided for Ed25519 signature_version 3 proof".to_string(),
                        ))
                    }
                };
                let payload = self
                    .construct_v3_payload()
                    .map_err(|e| ContextraError::Internal(e.to_string()))?;

                use ed25519_dalek::Verifier;
                let sig = match ed25519_dalek::Signature::from_slice(&self.signature) {
                    Ok(s) => s,
                    Err(_) => return Ok(false),
                };

                Ok(verifying_key.verify(&payload, &sig).is_ok())
            }
            v => Err(ContextraError::Internal(format!(
                "Unsupported DeletionProof signature_version: {v}"
            ))),
        }
    }

    /// Helper to construct the signed payload for signature version 3 (Ed25519).
    fn construct_v3_payload(&self) -> std::result::Result<Vec<u8>, CryptoError> {
        let scope_bytes = bincode::serialize(&self.scope)
            .map_err(|e| CryptoError::Crypto(e.to_string()))?;
        let tx_bytes = self.deleted_after_tx.0.to_le_bytes();
        let timestamp_bytes = self.timestamp.to_le_bytes();
        let covered_layers_bytes = bincode::serialize(&self.covered_layers)
            .map_err(|e| CryptoError::Crypto(e.to_string()))?;
        let excluded_scopes_bytes = bincode::serialize(&self.excluded_scopes)
            .map_err(|e| CryptoError::Crypto(e.to_string()))?;
        let receipt_bytes = self.wal_chain_receipt.unwrap_or([0u8; 32]);
        let receipt_part = if self.wal_chain_receipt.is_some() {
            receipt_bytes.as_slice()
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
                + receipt_part.len(),
        );
        payload.extend_from_slice(&scope_bytes);
        payload.extend_from_slice(&self.deleted_keys_hash);
        payload.extend_from_slice(&tx_bytes);
        payload.extend_from_slice(&timestamp_bytes);
        payload.extend_from_slice(&covered_layers_bytes);
        payload.extend_from_slice(&excluded_scopes_bytes);
        payload.extend_from_slice(receipt_part);

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
    /// # Errors
    /// - [`CryptoError::UnsupportedProofVersion`] if `signature_version` is not 3.
    /// - [`CryptoError::InvalidProofSignature`] if the signature does not match or cannot be parsed.
    pub fn verify_external(
        &self,
        public_key: &ed25519_dalek::VerifyingKey,
    ) -> std::result::Result<(), CryptoError> {
        if self.signature_version != 3 {
            return Err(CryptoError::UnsupportedProofVersion(self.signature_version));
        }

        let payload = self.construct_v3_payload()?;

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
        if self.signature_version == 1 {
            let mut clone = self.clone();
            clone.integrity_warning = Some(
                "covered_layers/excluded_scopes are not cryptographically signed in this legacy proof version"
                    .to_string(),
            );
            serde_json::to_string_pretty(&clone)
                .map_err(|e| ContextraError::Internal(e.to_string()))
        } else {
            serde_json::to_string_pretty(self).map_err(|e| ContextraError::Internal(e.to_string()))
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
        .map_err(|e| ContextraError::Internal(format!("HMAC key error: {e}")))?;
    for part in data_parts {
        mac.update(part);
    }
    Ok(mac.finalize().into_bytes().into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use contextra_types::{DocId, TenantId, TxId};

    fn test_key() -> Vec<u8> {
        vec![0u8; 32]
    }

    #[test]
    fn test_hash_collision_ab_c_vs_a_bc() {
        let keys1 = vec![b"ab".to_vec(), b"c".to_vec()];
        let keys2 = vec![b"a".to_vec(), b"bc".to_vec()];
        assert_ne!(
            hash_deleted_keys_length_prefixed(&keys1),
            hash_deleted_keys_length_prefixed(&keys2)
        );
    }

    #[test]
    fn test_v3_create_and_verify() {
        let keypair = DeletionProofKeyPair::generate();
        let scope = DeletionScope::Document {
            doc_id: DocId(42),
            tenant_id: TenantId::try_new(1).unwrap(),
        };
        let proof = DeletionProof::create_v3(
            scope,
            vec![b"k1".to_vec(), b"k2".to_vec()],
            TxId(100),
            vec![
                LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 0).unwrap(),
            ],
            vec![ExcludedScope::LlmParameterMemory],
            keypair.signing_key(),
        )
        .unwrap();

        assert_eq!(proof.signature_version, 3);
        assert_eq!(proof.signature.len(), 64);
        assert!(proof.verify(&keypair.verifying_key).unwrap());
    }

    #[test]
    fn test_v3_wrong_verifying_key_rejects() {
        let keypair1 = DeletionProofKeyPair::generate();
        let keypair2 = DeletionProofKeyPair::generate();

        let scope = DeletionScope::Tenant {
            tenant_id: TenantId::try_new(1).unwrap(),
        };
        let proof = DeletionProof::create_v3(
            scope,
            vec![b"k1".to_vec()],
            TxId(10),
            vec![],
            vec![],
            keypair1.signing_key(),
        )
        .unwrap();

        assert!(!proof.verify(&keypair2.verifying_key).unwrap());
    }

    #[test]
    fn test_v3_tampered_signature_rejects() {
        let keypair = DeletionProofKeyPair::generate();
        let scope = DeletionScope::Tenant {
            tenant_id: TenantId::try_new(1).unwrap(),
        };
        let mut proof = DeletionProof::create_v3(
            scope,
            vec![b"k1".to_vec()],
            TxId(10),
            vec![],
            vec![],
            keypair.signing_key(),
        )
        .unwrap();

        proof.signature[0] ^= 0xFF;
        assert!(!proof.verify(&keypair.verifying_key).unwrap());
    }

    #[test]
    fn test_v3_tampered_payload_rejects() {
        let keypair = DeletionProofKeyPair::generate();
        let scope = DeletionScope::Document {
            doc_id: DocId(42),
            tenant_id: TenantId::try_new(1).unwrap(),
        };
        let proof = DeletionProof::create_with_wal_receipt_v3(
            scope,
            vec![b"k1".to_vec()],
            TxId(10),
            vec![
                LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 0).unwrap(),
            ],
            vec![ExcludedScope::LlmParameterMemory],
            Some([0xABu8; 32]),
            keypair.signing_key(),
        )
        .unwrap();

        assert!(proof.verify(&keypair.verifying_key).unwrap());

        // Scope
        let mut tampered = proof.clone();
        tampered.scope = DeletionScope::Document {
            doc_id: DocId(43),
            tenant_id: TenantId::try_new(1).unwrap(),
        };
        assert!(!tampered.verify(&keypair.verifying_key).unwrap());

        // Keys hash
        let mut tampered = proof.clone();
        tampered.deleted_keys_hash[0] ^= 0xFF;
        assert!(!tampered.verify(&keypair.verifying_key).unwrap());

        // TxId
        let mut tampered = proof.clone();
        tampered.deleted_after_tx = TxId(11);
        assert!(!tampered.verify(&keypair.verifying_key).unwrap());

        // Covered layers
        let mut tampered = proof.clone();
        tampered.covered_layers.push(DeletionLayer::HnswIndex);
        assert!(!tampered.verify(&keypair.verifying_key).unwrap());

        // Excluded scopes
        let mut tampered = proof.clone();
        tampered.excluded_scopes.clear();
        assert!(!tampered.verify(&keypair.verifying_key).unwrap());

        // WAL receipt
        let mut tampered = proof.clone();
        tampered.wal_chain_receipt = Some([0xCDu8; 32]);
        assert!(!tampered.verify(&keypair.verifying_key).unwrap());
    }

    #[test]
    fn test_v3_cannot_verify_with_hmac_key() {
        let keypair = DeletionProofKeyPair::generate();
        let scope = DeletionScope::Tenant {
            tenant_id: TenantId::try_new(1).unwrap(),
        };
        let proof = DeletionProof::create_v3(
            scope,
            vec![b"k1".to_vec()],
            TxId(10),
            vec![],
            vec![],
            keypair.signing_key(),
        )
        .unwrap();

        let hmac_key = vec![0u8; 32];
        let res = proof.verify(&hmac_key);
        assert!(res.is_err());
        match res {
            Err(ContextraError::Internal(msg)) => {
                assert!(msg.contains("HMAC key provided for Ed25519 signature_version 3 proof"));
            }
            _ => panic!("Expected ContextraError::Internal"),
        }
    }

    #[test]
    fn test_v1_v2_still_verify_after_v3_code() {
        // Run existing v1 & v2 tests logic to ensure zero regression
        let scope = DeletionScope::Tenant {
            tenant_id: TenantId::try_new(1).unwrap(),
        };

        // v2
        let proof_v2 = DeletionProof::create(
            scope.clone(),
            vec![b"k1".to_vec()],
            TxId(10),
            vec![
                LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 0).unwrap(),
            ],
            vec![ExcludedScope::LlmParameterMemory],
            &test_key(),
        )
        .unwrap();
        assert_eq!(proof_v2.signature_version, 2);
        assert!(proof_v2.verify(&test_key()).unwrap());

        // v1
        let scope_bytes = bincode::serialize(&scope).unwrap();
        let deleted_keys_hash = [0u8; 32];
        let tx_bytes = TxId(10).0.to_le_bytes();
        let v1_signature =
            compute_hmac_sha256(&test_key(), &[&scope_bytes, &deleted_keys_hash, &tx_bytes])
                .unwrap();

        let v1_proof = DeletionProof {
            signature_version: 1,
            scope,
            deleted_keys_hash,
            deleted_after_tx: TxId(10),
            timestamp: 0,
            signature: v1_signature.to_vec(),
            covered_layers: vec![],
            excluded_scopes: vec![],
            wal_chain_receipt: None,
            integrity_warning: None,
        };
        assert_eq!(v1_proof.signature_version, 1);
        assert!(v1_proof.verify(&test_key()).unwrap());
    }

    #[test]
    fn test_v3_uses_length_prefixed_hash() {
        let keypair = DeletionProofKeyPair::generate();
        let scope = DeletionScope::Tenant {
            tenant_id: TenantId::try_new(1).unwrap(),
        };

        let proof_ab_c = DeletionProof::create_v3(
            scope.clone(),
            vec![b"ab".to_vec(), b"c".to_vec()],
            TxId(10),
            vec![],
            vec![],
            keypair.signing_key(),
        )
        .unwrap();

        let proof_a_bc = DeletionProof::create_v3(
            scope,
            vec![b"a".to_vec(), b"bc".to_vec()],
            TxId(10),
            vec![],
            vec![],
            keypair.signing_key(),
        )
        .unwrap();

        assert_ne!(proof_ab_c.deleted_keys_hash, proof_a_bc.deleted_keys_hash);
    }

    #[test]
    fn test_deletion_proof_create_and_verify() {
        let tenant = TenantId::try_new(1).unwrap();
        let scope = DeletionScope::Document {
            doc_id: DocId(42),
            tenant_id: tenant,
        };
        let keys = vec![b"key1".to_vec(), b"key2".to_vec()];
        let proof = DeletionProof::create(
            scope,
            keys,
            TxId(100),
            vec![
                LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 0).unwrap(),
                LayerCleanupProof::new_after_verified_empty(DeletionLayer::HnswIndex, 0).unwrap(),
            ],
            vec![ExcludedScope::LlmParameterMemory],
            &test_key(),
        )
        .unwrap();

        assert!(proof.verify(&test_key()).unwrap());
    }

    #[test]
    fn test_layer_cleanup_proof_rejects_nonzero_remaining_entries() {
        let res = LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 3);
        assert!(res.is_err());
        match res {
            Err(ContextraError::Internal(msg)) => {
                assert!(msg.contains("INV-DELETION-1 violation"));
                assert!(msg.contains("found 3 remaining live entries"));
            }
            _ => panic!("Expected ContextraError::Internal"),
        }
    }

    #[test]
    fn test_deletion_proof_wrong_key_fails_verify() {
        let scope = DeletionScope::Tenant {
            tenant_id: TenantId::try_new(1).unwrap(),
        };
        let proof =
            DeletionProof::create(scope, vec![], TxId(1), vec![], vec![], &test_key()).unwrap();

        let wrong_key = vec![1u8; 32];
        assert!(!proof.verify(&wrong_key).unwrap());
    }

    #[test]
    fn test_deletion_proof_tampered_data_fails_verify() {
        let scope = DeletionScope::Tenant {
            tenant_id: TenantId::try_new(1).unwrap(),
        };
        let proof = DeletionProof::create(
            scope,
            vec![b"k1".to_vec()],
            TxId(10),
            vec![
                LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 0).unwrap(),
            ],
            vec![ExcludedScope::LlmParameterMemory],
            &test_key(),
        )
        .unwrap();

        // Tamper with deleted_after_tx
        let mut tampered_tx = proof.clone();
        tampered_tx.deleted_after_tx = TxId(11);
        assert!(!tampered_tx.verify(&test_key()).unwrap());

        // Tamper with deleted_keys_hash
        let mut tampered_hash = proof.clone();
        tampered_hash.deleted_keys_hash[0] ^= 0xFF;
        assert!(!tampered_hash.verify(&test_key()).unwrap());

        // Tamper with scope
        let mut tampered_scope = proof.clone();
        tampered_scope.scope = DeletionScope::Tenant {
            tenant_id: TenantId::try_new(2).unwrap(),
        };
        assert!(!tampered_scope.verify(&test_key()).unwrap());

        // Untampered original must verify successfully
        assert!(proof.verify(&test_key()).unwrap());
    }

    #[test]
    fn test_deletion_proof_key_order_deterministic() {
        // Gleiche Keys in anderer Reihenfolge → gleicher Hash (weil sortiert)
        let keys_a = vec![b"b".to_vec(), b"a".to_vec()];
        let keys_b = vec![b"a".to_vec(), b"b".to_vec()];

        let proof_a = DeletionProof::create(
            DeletionScope::Tenant {
                tenant_id: TenantId::try_new(2).unwrap(),
            },
            keys_a,
            TxId(1),
            vec![],
            vec![],
            &test_key(),
        )
        .unwrap();
        let proof_b = DeletionProof::create(
            DeletionScope::Tenant {
                tenant_id: TenantId::try_new(2).unwrap(),
            },
            keys_b,
            TxId(1),
            vec![],
            vec![],
            &test_key(),
        )
        .unwrap();

        assert_eq!(proof_a.deleted_keys_hash, proof_b.deleted_keys_hash);
    }

    #[test]
    fn test_deletion_proof_audit_export_contains_excluded_scopes() {
        let scope = DeletionScope::Tenant {
            tenant_id: TenantId::try_new(1).unwrap(),
        };
        let proof = DeletionProof::create(
            scope,
            vec![],
            TxId(1),
            vec![],
            vec![
                ExcludedScope::LlmParameterMemory,
                ExcludedScope::ConsolidatedAndDistilled,
            ],
            &test_key(),
        )
        .unwrap();

        let json = proof.export_for_audit().unwrap();
        assert!(json.contains("LlmParameterMemory"));
        assert!(json.contains("ConsolidatedAndDistilled"));
    }

    #[test]
    fn test_deletion_proof_tenant_id_extraction() {
        let t1 = TenantId::try_new(10).unwrap();
        let doc_scope = DeletionScope::Document {
            doc_id: DocId(1),
            tenant_id: t1,
        };
        let p1 =
            DeletionProof::create(doc_scope, vec![], TxId(1), vec![], vec![], &test_key()).unwrap();
        assert_eq!(p1.tenant_id(), t1);

        let t2 = TenantId::try_new(20).unwrap();
        let col_scope = DeletionScope::Collection {
            collection_id: CollectionId(5),
            tenant_id: t2,
        };
        let p2 =
            DeletionProof::create(col_scope, vec![], TxId(1), vec![], vec![], &test_key()).unwrap();
        assert_eq!(p2.tenant_id(), t2);

        let t3 = TenantId::try_new(30).unwrap();
        let tenant_scope = DeletionScope::Tenant { tenant_id: t3 };
        let p3 = DeletionProof::create(tenant_scope, vec![], TxId(1), vec![], vec![], &test_key())
            .unwrap();
        assert_eq!(p3.tenant_id(), t3);
    }

    #[test]
    fn test_deletion_proof_requires_layer_cleanup_proof() {
        let scope = DeletionScope::Tenant {
            tenant_id: TenantId::try_new(100).unwrap(),
        };

        let cleanup_proofs = vec![
            LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 0).unwrap(),
            LayerCleanupProof::new_after_verified_empty(DeletionLayer::SsTableAllLevels, 0)
                .unwrap(),
            LayerCleanupProof::new_after_verified_empty(DeletionLayer::HnswIndex, 0).unwrap(),
        ];

        let proof = DeletionProof::create(
            scope,
            vec![b"k1".to_vec()],
            TxId(1),
            cleanup_proofs,
            vec![],
            &test_key(),
        )
        .unwrap();

        assert!(proof.verify(&test_key()).unwrap());
        assert_eq!(
            proof.covered_layers,
            vec![
                DeletionLayer::LsmMemtable,
                DeletionLayer::SsTableAllLevels,
                DeletionLayer::HnswIndex,
            ]
        );
    }

    #[test]
    fn test_deletion_proof_tampered_covered_layers_fails_verify() {
        let scope = DeletionScope::Tenant {
            tenant_id: TenantId::try_new(1).unwrap(),
        };
        let proof = DeletionProof::create(
            scope,
            vec![b"k1".to_vec()],
            TxId(10),
            vec![
                LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 0).unwrap(),
            ],
            vec![ExcludedScope::LlmParameterMemory],
            &test_key(),
        )
        .unwrap();

        assert_eq!(proof.signature_version, 2);
        assert!(proof.verify(&test_key()).unwrap());

        let mut tampered_layers = proof.clone();
        tampered_layers
            .covered_layers
            .push(DeletionLayer::KvCacheSegments);

        assert!(!tampered_layers.verify(&test_key()).unwrap());
    }

    #[test]
    fn test_deletion_proof_tampered_excluded_scopes_fails_verify() {
        let scope = DeletionScope::Tenant {
            tenant_id: TenantId::try_new(1).unwrap(),
        };
        let proof = DeletionProof::create(
            scope,
            vec![b"k1".to_vec()],
            TxId(10),
            vec![
                LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 0).unwrap(),
            ],
            vec![
                ExcludedScope::LlmParameterMemory,
                ExcludedScope::ConsolidatedAndDistilled,
            ],
            &test_key(),
        )
        .unwrap();

        assert_eq!(proof.signature_version, 2);
        assert!(proof.verify(&test_key()).unwrap());

        let mut tampered_scopes = proof.clone();
        tampered_scopes.excluded_scopes.pop();

        assert!(!tampered_scopes.verify(&test_key()).unwrap());
    }

    #[test]
    fn test_deletion_proof_v1_backward_compatibility() {
        let scope = DeletionScope::Tenant {
            tenant_id: TenantId::try_new(1).unwrap(),
        };
        let deleted_after_tx = TxId(10);
        let deleted_keys = vec![b"k1".to_vec()];

        // Construct a v1 signature manually using only the 3 legacy fields
        let mut hasher = blake3::Hasher::new();
        for key in &deleted_keys {
            hasher.update(key);
        }
        let deleted_keys_hash: [u8; 32] = *hasher.finalize().as_bytes();

        let scope_bytes = bincode::serialize(&scope).unwrap();
        let tx_bytes = deleted_after_tx.0.to_le_bytes();

        let v1_signature =
            compute_hmac_sha256(&test_key(), &[&scope_bytes, &deleted_keys_hash, &tx_bytes])
                .unwrap();

        let v1_proof = DeletionProof {
            signature_version: 1,
            scope,
            deleted_keys_hash,
            deleted_after_tx,
            timestamp: 0,
            signature: v1_signature.to_vec(),
            covered_layers: vec![DeletionLayer::LsmMemtable],
            excluded_scopes: vec![ExcludedScope::LlmParameterMemory],
            wal_chain_receipt: None,
            integrity_warning: None,
        };

        // v1 proof verifies successfully with original 3 fields intact
        assert!(v1_proof.verify(&test_key()).unwrap());

        // Test serde deserialization of JSON missing signature_version defaults to 1
        let json_missing_version = r#"{
            "scope": {"Tenant": {"tenant_id": 1}},
            "deleted_keys_hash": "#
            .to_string()
            + &serde_json::to_string(&deleted_keys_hash).unwrap()
            + r#",
            "deleted_after_tx": 10,
            "signature": "#
            + &serde_json::to_string(&v1_signature).unwrap()
            + r#",
            "covered_layers": ["LsmMemtable"],
            "excluded_scopes": ["LlmParameterMemory"]
        }"#;

        let deserialized: DeletionProof = serde_json::from_str(&json_missing_version).unwrap();
        assert_eq!(deserialized.signature_version, 1);
        assert!(deserialized.verify(&test_key()).unwrap());
    }

    #[test]
    fn test_deletion_proof_v1_audit_export_integrity_warning() {
        let scope = DeletionScope::Tenant {
            tenant_id: TenantId::try_new(1).unwrap(),
        };
        let deleted_after_tx = TxId(10);
        let deleted_keys = vec![b"k1".to_vec()];

        let mut hasher = blake3::Hasher::new();
        for key in &deleted_keys {
            hasher.update(key);
        }
        let deleted_keys_hash: [u8; 32] = *hasher.finalize().as_bytes();

        let scope_bytes = bincode::serialize(&scope).unwrap();
        let tx_bytes = deleted_after_tx.0.to_le_bytes();

        let v1_signature =
            compute_hmac_sha256(&test_key(), &[&scope_bytes, &deleted_keys_hash, &tx_bytes])
                .unwrap();

        let v1_proof = DeletionProof {
            signature_version: 1,
            scope,
            deleted_keys_hash,
            deleted_after_tx,
            timestamp: 0,
            signature: v1_signature.to_vec(),
            covered_layers: vec![DeletionLayer::LsmMemtable],
            excluded_scopes: vec![ExcludedScope::LlmParameterMemory],
            wal_chain_receipt: None,
            integrity_warning: None,
        };

        let json = v1_proof.export_for_audit().unwrap();
        assert!(
            json.contains("integrity_warning"),
            "v1 export MUST contain integrity_warning"
        );
        assert!(
            json.contains(
                "covered_layers/excluded_scopes are not cryptographically signed in this legacy proof version"
            ),
            "v1 export MUST contain the exact warning message"
        );
    }

    #[test]
    fn test_deletion_proof_v2_audit_export_no_integrity_warning() {
        let scope = DeletionScope::Tenant {
            tenant_id: TenantId::try_new(1).unwrap(),
        };
        let proof = DeletionProof::create(
            scope,
            vec![b"k1".to_vec()],
            TxId(10),
            vec![
                LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 0).unwrap(),
            ],
            vec![ExcludedScope::LlmParameterMemory],
            &test_key(),
        )
        .unwrap();

        assert_eq!(proof.signature_version, 2);
        let json = proof.export_for_audit().unwrap();
        assert!(
            !json.contains("integrity_warning"),
            "v2 export MUST NOT contain integrity_warning"
        );
    }

    #[test]
    fn test_wal_delete_receipt_computation_and_o1_verification() {
        let integrity_key = b"integrity-key-32-bytes-wal-rec!";
        let prev_hmac = [0x55u8; 32];
        let delete_event = b"delete_event:doc_id=42:tx_id=100";

        let receipt = compute_wal_delete_receipt(&prev_hmac, delete_event, integrity_key).unwrap();

        // O(1) Verification without cleartext
        assert!(
            verify_wal_delete_receipt(&receipt, &prev_hmac, delete_event, integrity_key).unwrap()
        );

        // Tampered receipt or wrong key fails
        let wrong_key = b"wrong-integrity-key-32-bytes---";
        assert!(!verify_wal_delete_receipt(&receipt, &prev_hmac, delete_event, wrong_key).unwrap());

        let tampered_event = b"delete_event:doc_id=43:tx_id=100";
        assert!(
            !verify_wal_delete_receipt(&receipt, &prev_hmac, tampered_event, integrity_key)
                .unwrap()
        );
    }

    #[test]
    fn test_deletion_proof_with_wal_receipt_creation_and_tamper_check() {
        let scope = DeletionScope::Document {
            doc_id: DocId(42),
            tenant_id: TenantId::try_new(1).unwrap(),
        };
        let integrity_key = b"integrity-key-32-bytes-wal-rec!";
        let prev_hmac = [0x77u8; 32];
        let delete_event = b"doc_42_delete";

        let receipt = compute_wal_delete_receipt(&prev_hmac, delete_event, integrity_key).unwrap();

        let proof = DeletionProof::create_with_wal_receipt(
            scope,
            vec![b"k42".to_vec()],
            TxId(100),
            vec![
                LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 0).unwrap(),
            ],
            vec![ExcludedScope::LlmParameterMemory],
            Some(receipt),
            &test_key(),
        )
        .unwrap();

        assert_eq!(proof.wal_chain_receipt, Some(receipt));
        assert!(proof.verify(&test_key()).unwrap());

        // Tamper with receipt
        let mut tampered_proof = proof.clone();
        tampered_proof.wal_chain_receipt = Some([0xFFu8; 32]);
        assert!(!tampered_proof.verify(&test_key()).unwrap());
    }

    #[test]
    fn test_layer_cleanup_proof_verify_and_create() {
        // Success path: closure returns Ok(true)
        let proof_ok = LayerCleanupProof::verify_and_create(DeletionLayer::HnswIndex, || Ok(true));
        assert!(proof_ok.is_ok());
        assert_eq!(proof_ok.unwrap().layer(), &DeletionLayer::HnswIndex);

        // Verification failed path: closure returns Ok(false)
        let proof_fail =
            LayerCleanupProof::verify_and_create(DeletionLayer::HnswIndex, || Ok(false));
        assert!(
            matches!(proof_fail, Err(ContextraError::Internal(ref msg)) if msg.contains("verification failed"))
        );

        // Closure returns error
        let proof_err = LayerCleanupProof::verify_and_create(DeletionLayer::HnswIndex, || {
            Err(ContextraError::Internal("db error".to_string()))
        });
        assert!(matches!(proof_err, Err(ContextraError::Internal(ref msg)) if msg == "db error"));
    }

    #[test]
    fn test_verify_external_v3_valid_and_invalid_keys() {
        let keypair = DeletionProofKeyPair::generate();
        let scope = DeletionScope::Document {
            doc_id: DocId(42),
            tenant_id: TenantId::try_new(1).unwrap(),
        };
        let proof = DeletionProof::create_v3(
            scope,
            vec![b"k1".to_vec()],
            TxId(100),
            vec![
                LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 0).unwrap(),
            ],
            vec![ExcludedScope::LlmParameterMemory],
            keypair.signing_key(),
        )
        .unwrap();

        // Valid verifying key
        assert!(proof.verify_external(&keypair.verifying_key).is_ok());

        // Invalid verifying key
        let wrong_keypair = DeletionProofKeyPair::generate();
        assert!(proof.verify_external(&wrong_keypair.verifying_key).is_err());
    }

    #[test]
    fn test_verify_external_v2_hmac() {
        let keypair = DeletionProofKeyPair::generate();
        let scope = DeletionScope::Tenant {
            tenant_id: TenantId::try_new(1).unwrap(),
        };
        let hmac_key = b"secret_hmac_key_32_bytes_long!!";
        let proof = DeletionProof::create(
            scope,
            vec![b"k1".to_vec()],
            TxId(10),
            vec![],
            vec![],
            hmac_key,
        )
        .unwrap();

        let keypair = DeletionProofKeyPair::generate();
        assert!(proof.verify_external(&keypair.verifying_key).is_err());
    }

    #[test]
    fn test_deletion_proof_unsupported_signature_version() {
        let scope = DeletionScope::Tenant {
            tenant_id: TenantId::try_new(1).unwrap(),
        };
        let mut proof =
            DeletionProof::create(scope, vec![], TxId(1), vec![], vec![], &test_key()).unwrap();

        proof.signature_version = 255;
        let res = proof.verify(&test_key());
        assert!(
            matches!(res, Err(ContextraError::Internal(ref msg)) if msg.contains("Unsupported DeletionProof signature_version: 255"))
        );
    }

    #[test]
    fn test_verify_external_v3_success() {
        let keypair = DeletionProofKeyPair::generate();
        let scope = DeletionScope::Document {
            doc_id: DocId(42),
            tenant_id: TenantId::try_new(1).unwrap(),
        };
        let proof = DeletionProof::create_v3(
            scope,
            vec![b"k1".to_vec(), b"k2".to_vec()],
            TxId(100),
            vec![
                LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 0).unwrap(),
            ],
            vec![ExcludedScope::LlmParameterMemory],
            keypair.signing_key(),
        )
        .unwrap();

        assert!(proof.verify_external(&keypair.verifying_key).is_ok());
    }

    #[test]
    fn test_verify_external_v3_tampered_payload_returns_invalid_proof_signature() {
        let keypair = DeletionProofKeyPair::generate();
        let scope = DeletionScope::Document {
            doc_id: DocId(42),
            tenant_id: TenantId::try_new(1).unwrap(),
        };
        let proof = DeletionProof::create_with_wal_receipt_v3(
            scope,
            vec![b"k1".to_vec()],
            TxId(10),
            vec![
                LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 0).unwrap(),
            ],
            vec![ExcludedScope::LlmParameterMemory],
            Some([0xABu8; 32]),
            keypair.signing_key(),
        )
        .unwrap();

        // 1. Tamper timestamp / TxId
        let mut tampered = proof.clone();
        tampered.deleted_after_tx = TxId(11);
        assert!(matches!(
            tampered.verify_external(&keypair.verifying_key),
            Err(CryptoError::InvalidProofSignature)
        ));

        // 2. Tamper deleted_keys_hash
        let mut tampered = proof.clone();
        tampered.deleted_keys_hash[0] ^= 0xFF;
        assert!(matches!(
            tampered.verify_external(&keypair.verifying_key),
            Err(CryptoError::InvalidProofSignature)
        ));

        // 3. Tamper scope
        let mut tampered = proof.clone();
        tampered.scope = DeletionScope::Document {
            doc_id: DocId(43),
            tenant_id: TenantId::try_new(1).unwrap(),
        };
        assert!(matches!(
            tampered.verify_external(&keypair.verifying_key),
            Err(CryptoError::InvalidProofSignature)
        ));

        // 4. Tamper covered_layers
        let mut tampered = proof.clone();
        tampered.covered_layers.push(DeletionLayer::HnswIndex);
        assert!(matches!(
            tampered.verify_external(&keypair.verifying_key),
            Err(CryptoError::InvalidProofSignature)
        ));

        // 5. Tamper excluded_scopes
        let mut tampered = proof.clone();
        tampered.excluded_scopes.clear();
        assert!(matches!(
            tampered.verify_external(&keypair.verifying_key),
            Err(CryptoError::InvalidProofSignature)
        ));

        // 6. Tamper wal_chain_receipt
        let mut tampered = proof.clone();
        tampered.wal_chain_receipt = Some([0xCDu8; 32]);
        assert!(matches!(
            tampered.verify_external(&keypair.verifying_key),
            Err(CryptoError::InvalidProofSignature)
        ));

        // 7. Tamper signature bytes
        let mut tampered = proof.clone();
        tampered.signature[0] ^= 0xFF;
        assert!(matches!(
            tampered.verify_external(&keypair.verifying_key),
            Err(CryptoError::InvalidProofSignature)
        ));

        // 8. Invalid public key
        let wrong_keypair = DeletionProofKeyPair::generate();
        assert!(matches!(
            proof.verify_external(&wrong_keypair.verifying_key),
            Err(CryptoError::InvalidProofSignature)
        ));

        // 9. Malformed signature length
        let mut tampered = proof.clone();
        tampered.signature.truncate(32);
        assert!(matches!(
            tampered.verify_external(&keypair.verifying_key),
            Err(CryptoError::InvalidProofSignature)
        ));
    }

    #[test]
    fn test_verify_external_v1_v2_unsupported_version_error() {
        let keypair = DeletionProofKeyPair::generate();
        let scope = DeletionScope::Tenant {
            tenant_id: TenantId::try_new(1).unwrap(),
        };

        // v2 HMAC proof
        let proof_v2 = DeletionProof::create(
            scope.clone(),
            vec![b"k1".to_vec()],
            TxId(10),
            vec![],
            vec![],
            &test_key(),
        )
        .unwrap();
        assert_eq!(proof_v2.signature_version, 2);
        assert!(matches!(
            proof_v2.verify_external(&keypair.verifying_key),
            Err(CryptoError::UnsupportedProofVersion(2))
        ));

        // v1 HMAC proof
        let mut proof_v1 = proof_v2.clone();
        proof_v1.signature_version = 1;
        assert!(matches!(
            proof_v1.verify_external(&keypair.verifying_key),
            Err(CryptoError::UnsupportedProofVersion(1))
        ));
    }
}
