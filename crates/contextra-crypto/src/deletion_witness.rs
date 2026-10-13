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
//!     let _ = (*witness).clone();
//! }
//! ```

#![forbid(unsafe_code)]

use crate::deletion_proof::{
    claim_text::{WitAssurance, WitClaimText},
    evidence::WitLayerEvidence,
    hash_deleted_keys_length_prefixed,
    probe::WitStoreGeneration,
    DeletionLayer,
};
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

/// Argumente für die Erstellung eines `LayerWitness` v2 über [`LayerWitness::issue`].
#[derive(Debug, Clone)]
pub struct WitIssueArgs {
    /// Betroffene Storage-Schicht.
    pub layer: DeletionLayer,
    /// BLAKE3 Hash der gelöschten Schlüssel.
    pub scope_hash: [u8; 32],
    /// Transaktions-ID zum Zeitpunkt des Scans.
    pub scanned_at_tx: TxId,
    /// Snapshot-Transaktions-ID der Löschforderung.
    pub as_of_tx: TxId,
    /// Assurance-Stufe der Löschung.
    pub assurance: WitAssurance,
    /// Zugesicherter Claim-Text.
    pub claim: WitClaimText,
    /// Footprint-Hash der betroffenen Datenbereiche.
    pub footprint_hash: [u8; 32],
    /// Schreib-Sequenznummer der Schicht.
    pub scope_write_seq: u64,
    /// Name des Scanners.
    pub scanner_name: &'static str,
    /// Version des Scanners.
    pub scanner_version: u32,
    /// Store Generation Token.
    pub store_generation: WitStoreGeneration,
    /// Detaillierte Schicht-Evidenz.
    pub layer_evidence: WitLayerEvidence,
    /// Ob auch Schlüsselbezeichner als frei beansprucht werden.
    pub key_identifiers_claimed: bool,
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
    assurance: WitAssurance,
    claim: WitClaimText,
    footprint_hash: [u8; 32],
    as_of_tx: TxId,
    scope_write_seq: u64,
    scanner: (&'static str, u32),
    store_generation: WitStoreGeneration,
    layer_evidence: Option<WitLayerEvidence>,
    _sealed: (),
}

impl LayerWitness {
    /// Erzeugt einen neuen `LayerWitness` unter Prüfung aller Schicht-Invarianten.
    ///
    /// # Errors
    /// Returns [`ContextraError::InvalidInput`] if:
    /// - `scanned_at_tx < as_of_tx`
    /// - `claim` does not match `assurance`
    /// - `layer_evidence` fails invariant checks
    #[doc(hidden)]
    pub fn issue(args: WitIssueArgs) -> Result<Self> {
        if args.scanned_at_tx < args.as_of_tx {
            return Err(ContextraError::InvalidInput(format!(
                "LayerWitness creation rejected: scanned_at_tx ({:?}) < as_of_tx ({:?})",
                args.scanned_at_tx, args.as_of_tx
            )));
        }

        // Claim and Assurance alignment validation
        let is_s2k = args.claim == WitClaimText::S2K;
        let key_identifiers_claimed = args.key_identifiers_claimed || is_s2k;

        match args.claim {
            WitClaimText::S0 => {}
            WitClaimText::S1 => {
                if args.assurance < WitAssurance::S1 {
                    return Err(ContextraError::InvalidInput(format!(
                        "LayerWitness creation rejected: claim S1 requires assurance >= S1, got {:?}",
                        args.assurance
                    )));
                }
            }
            WitClaimText::S2V | WitClaimText::S2K => {
                if args.assurance < WitAssurance::S2 {
                    return Err(ContextraError::InvalidInput(format!(
                        "LayerWitness creation rejected: claim {:?} requires assurance >= S2, got {:?}",
                        args.claim, args.assurance
                    )));
                }
            }
            WitClaimText::S3 => {
                if args.assurance < WitAssurance::S3 {
                    return Err(ContextraError::InvalidInput(format!(
                        "LayerWitness creation rejected: claim S3 requires assurance >= S3, got {:?}",
                        args.assurance
                    )));
                }
            }
            WitClaimText::S4 => {
                if args.assurance < WitAssurance::S4 {
                    return Err(ContextraError::InvalidInput(format!(
                        "LayerWitness creation rejected: claim S4 requires assurance >= S4, got {:?}",
                        args.assurance
                    )));
                }
            }
            WitClaimText::RamUnreferenced | WitClaimText::RamZeroized => {}
        }

        args.layer_evidence
            .check_invariants(args.assurance, key_identifiers_claimed)
            .map_err(|e| ContextraError::InvalidInput(e.to_string()))?;

        let legacy_evidence = match &args.layer_evidence {
            WitLayerEvidence::Memtable { memtables_scanned, .. } => WitnessEvidence::MemtableScan {
                memtables_scanned: *memtables_scanned,
            },
            WitLayerEvidence::SsTables { files_scanned, .. } => WitnessEvidence::SstableScan {
                files_scanned: *files_scanned,
            },
            WitLayerEvidence::Wal {
                segments_removed,
                bytes_scrubbed,
                ..
            } => WitnessEvidence::WalPurge {
                segments_removed: *segments_removed,
                bytes_scrubbed: *bytes_scrubbed,
            },
            _ => WitnessEvidence::MemtableScan {
                memtables_scanned: 0,
            },
        };

        Ok(Self {
            layer: args.layer,
            scope_hash: args.scope_hash,
            scanned_at_tx: args.scanned_at_tx,
            evidence: legacy_evidence,
            assurance: args.assurance,
            claim: args.claim,
            footprint_hash: args.footprint_hash,
            as_of_tx: args.as_of_tx,
            scope_write_seq: args.scope_write_seq,
            scanner: (args.scanner_name, args.scanner_version),
            store_generation: args.store_generation,
            layer_evidence: Some(args.layer_evidence),
            _sealed: (),
        })
    }

    /// Erzeugt einen unfälschbaren LayerWitness für eine Storage-Schicht (Legacy-Konstruktor).
    ///
    /// Nur aus contextra-store/contextra-vector aufrufen; CI-Regel H-01 erzwingt das.
    ///
    /// # Errors
    /// Returns [`ContextraError::InvalidInput`] if:
    /// - `deleted_keys` is empty.
    /// - `layer` is one of the non-scanner layers (`HnswIndex`, `CsrGraph`, `KvCacheSegments`, `EmbeddingCache`, etc.).
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
            | DeletionLayer::EmbeddingCache
            | DeletionLayer::TextIndex
            | DeletionLayer::Relations
            | DeletionLayer::Manifest
            | DeletionLayer::CheckpointClones
            | DeletionLayer::ProcessCaches
            | DeletionLayer::Keyring
            | DeletionLayer::ErasureLedger => {
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

        let assurance = match layer {
            DeletionLayer::SsTableAllLevels => WitAssurance::S2,
            DeletionLayer::WalAllSegments { .. } => WitAssurance::S3,
            _ => WitAssurance::S0,
        };

        let claim = match assurance {
            WitAssurance::S2 => WitClaimText::S2V,
            WitAssurance::S3 => WitClaimText::S3,
            _ => WitClaimText::S0,
        };

        Ok(Self {
            layer,
            scope_hash,
            scanned_at_tx,
            evidence,
            assurance,
            claim,
            footprint_hash: [0u8; 32],
            as_of_tx: scanned_at_tx,
            scope_write_seq: 0,
            scanner: ("legacy_scanner", 1),
            store_generation: WitStoreGeneration(0),
            layer_evidence: None,
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

    /// Gibt die Assurance-Stufe des Belegs zurück.
    pub fn assurance(&self) -> WitAssurance {
        self.assurance
    }

    /// Gibt den zugesicherten ClaimText zurück.
    pub fn claim(&self) -> WitClaimText {
        self.claim
    }

    /// Gibt den Footprint-Hash zurück.
    pub fn footprint_hash(&self) -> &[u8; 32] {
        &self.footprint_hash
    }

    /// Gibt die Snapshot-Transaktions-ID (as_of_tx) zurück.
    pub fn as_of_tx(&self) -> TxId {
        self.as_of_tx
    }

    /// Gibt die Schreib-Sequenznummer der Schicht zurück.
    pub fn scope_write_seq(&self) -> u64 {
        self.scope_write_seq
    }

    /// Gibt (Scanner-Name, Scanner-Version) zurück.
    pub fn scanner(&self) -> (&'static str, u32) {
        self.scanner
    }

    /// Gibt die Store Generation zurück.
    pub fn store_generation(&self) -> WitStoreGeneration {
        self.store_generation
    }

    /// Gibt die detaillierte Schicht-Evidenz zurück, falls vorhanden.
    pub fn layer_evidence(&self) -> Option<&WitLayerEvidence> {
        self.layer_evidence.as_ref()
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
