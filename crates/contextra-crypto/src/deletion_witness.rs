//! Unfälschbarer Layer-Beleg für physische Löschungen in Storage-Schichten (W5-02).
//!
//! Nur aus contextra-store/contextra-vector aufrufen; CI-Regel H-01 erzwingt das.
//!
//! # Doctests (Compile Fail Verification)
//!
//! Struct-Literale von außerhalb der Modulgrenzen werden abgelehnt:
//! ```compile_fail
//! use contextra_crypto::deletion_witness::{LayerWitness, WitnessEvidence};
//! use contextra_crypto::deletion_proof::DeletionLayer;
//! use contextra_types::TxId;
//!
//! let witness = LayerWitness {
//!     layer: DeletionLayer::LsmMemtable,
//!     scope_hash: [0u8; 32],
//!     scanned_at_tx: TxId(1),
//!     evidence: WitnessEvidence::MemtableScan { memtables_scanned: 1 },
//!     _sealed: (),
//! };
//! ```
//!
//! Default-Konstruktion ist verboten:
//! ```compile_fail
//! use contextra_crypto::deletion_witness::LayerWitness;
//!
//! let witness = LayerWitness::default();
//! ```
//!
//! Deserialisierung ist verboten:
//! ```compile_fail
//! use contextra_crypto::deletion_witness::LayerWitness;
//!
//! let json = "{}";
//! let witness: LayerWitness = serde_json::from_str(json).unwrap();
//! ```
//!
//! Clone ist verboten:
//! ```compile_fail
//! use contextra_crypto::deletion_witness::LayerWitness;
//!
//! fn test_clone(witness: &LayerWitness) {
//!     let _ = witness.clone();
//! }
//! ```

#![forbid(unsafe_code)]

use crate::deletion_proof::{hash_deleted_keys_length_prefixed, DeletionLayer};
use contextra_types::{ContextraError, Result, TxId};
use serde::Serialize;

/// Nachweis-Evidenz der jeweiligen Storage-Schicht.
#[non_exhaustive]
#[derive(Debug, Serialize, PartialEq, Eq)]
pub enum WitnessEvidence {
    /// In-Memory Memtable Scan Evidenz.
    MemtableScan {
        /// Anzahl durchsuchter Memtables.
        memtables_scanned: u32,
    },
    /// Persistent SSTable File Scan Evidenz.
    SstableScan {
        /// Anzahl durchsuchter SSTable-Dateien.
        files_scanned: u32,
    },
    /// WAL Truncation / Zeroization Evidenz.
    WalPurge {
        /// Anzahl entfernter WAL-Segmente.
        segments_removed: u32,
        /// Anzahl bereinigter/bereinigter Bytes.
        bytes_scrubbed: u64,
    },
}

/// Unfälschbarer Beweistyp für eine physisch gescannte Löschung in einer Storage-Schicht.
///
/// Nur aus contextra-store/contextra-vector aufrufen; CI-Regel H-01 erzwingt das.
#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct LayerWitness {
    layer: DeletionLayer,
    scope_hash: [u8; 32],
    scanned_at_tx: TxId,
    evidence: WitnessEvidence,
    _sealed: (),
}

impl LayerWitness {
    /// Erzeugt einen unfälschbaren LayerWitness für eine Storage-Schicht.
    ///
    /// Nur aus contextra-store/contextra-vector aufrufen; CI-Regel H-01 erzwingt das.
    ///
    /// # Errors
    /// Returns [`ContextraError::InvalidInput`] if:
    /// - `deleted_keys` is empty.
    /// - `layer` is one of the four non-scanner layers (`HnswIndex`, `CsrGraph`, `KvCacheSegments`, `EmbeddingCache`).
    /// - `evidence` does not match the layer type (`LsmMemtable` -> `MemtableScan`, `SsTableAllLevels` -> `SstableScan`, `WalAllSegments` -> `WalPurge`).
    #[doc(hidden)]
    pub fn issue_from_storage_layer(
        layer: DeletionLayer,
        deleted_keys: &[Vec<u8>],
        scanned_at_tx: TxId,
        evidence: WitnessEvidence,
    ) -> Result<Self> {
        if deleted_keys.is_empty() {
            return Err(ContextraError::InvalidInput(
                "LayerWitness creation rejected: deleted_keys cannot be empty".to_string(),
            ));
        }

        match layer {
            DeletionLayer::HnswIndex
            | DeletionLayer::CsrGraph
            | DeletionLayer::KvCacheSegments
            | DeletionLayer::EmbeddingCache => {
                return Err(ContextraError::InvalidInput(format!(
                    "LayerWitness creation rejected: layer {:?} does not support scanner-based witness creation",
                    layer
                )));
            }
            DeletionLayer::LsmMemtable => {
                if !matches!(evidence, WitnessEvidence::MemtableScan { .. }) {
                    return Err(ContextraError::InvalidInput(format!(
                        "LayerWitness evidence mismatch: layer LsmMemtable requires MemtableScan evidence, got {:?}",
                        evidence
                    )));
                }
            }
            DeletionLayer::SsTableAllLevels => {
                if !matches!(evidence, WitnessEvidence::SstableScan { .. }) {
                    return Err(ContextraError::InvalidInput(format!(
                        "LayerWitness evidence mismatch: layer SsTableAllLevels requires SstableScan evidence, got {:?}",
                        evidence
                    )));
                }
            }
            DeletionLayer::WalAllSegments { .. } => {
                if !matches!(evidence, WitnessEvidence::WalPurge { .. }) {
                    return Err(ContextraError::InvalidInput(format!(
                        "LayerWitness evidence mismatch: layer WalAllSegments requires WalPurge evidence, got {:?}",
                        evidence
                    )));
                }
            }
        }

        let scope_hash = hash_deleted_keys_length_prefixed(deleted_keys);

        Ok(Self {
            layer,
            scope_hash,
            scanned_at_tx,
            evidence,
            _sealed: (),
        })
    }

    /// Gibt die referenzierte Storage-Schicht zurück.
    pub fn layer(&self) -> &DeletionLayer {
        &self.layer
    }

    /// Gibt den Blake3 Scope-Hash der gelöschten Schlüssel zurück.
    pub fn scope_hash(&self) -> &[u8; 32] {
        &self.scope_hash
    }

    /// Gibt die Transaktions-ID zum Zeitpunkt des Scans zurück.
    pub fn scanned_at_tx(&self) -> TxId {
        self.scanned_at_tx
    }

    /// Gibt die Evidenz-Metriken des Scans zurück.
    pub fn evidence(&self) -> &WitnessEvidence {
        &self.evidence
    }

    /// Verifiziert die Bindung des Belegs an den erwarteten Scope-Hash und Transaktions-Zeitpunkt.
    pub fn check_binding(
        &self,
        expected_scope_hash: &[u8; 32],
        deleted_after_tx: TxId,
    ) -> Result<()> {
        if self.scope_hash != *expected_scope_hash {
            return Err(ContextraError::InvalidInput(format!(
                "LayerWitness binding check failed: scope_hash mismatch (expected {:?}, got {:?})",
                expected_scope_hash, self.scope_hash
            )));
        }
        if self.scanned_at_tx < deleted_after_tx {
            return Err(ContextraError::InvalidInput(format!(
                "LayerWitness binding check failed: scanned_at_tx ({:?}) < deleted_after_tx ({:?})",
                self.scanned_at_tx, deleted_after_tx
            )));
        }
        Ok(())
    }
}
