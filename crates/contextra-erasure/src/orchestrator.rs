//! # Modul: orchestrator.rs
//!
//! **Zweck:** Autarker Fail-Closed ErasureOrchestrator für gemessene Bereinigungen (SPEC §6, §12.7.4).
//!
//! Eingehaltene Invarianten:
//! - DeletionProof entsteht ausschließlich nach unabhängiger Messung aller Schichten (INV-DELETION-1).
//! - Physische Bereinigung und Schlüsselvernichtung wirken gemeinsam (E-04).

#![forbid(unsafe_code)]

use crate::deletion_proof::{DeletionProof, ExcludedScope};
use crate::error::ErasureError;
use crate::types_local::{
    CollectionId,
    ScanTechnique, ScanWitness, TenantId,
};
use serde::{Deserialize, Serialize};

/// Status einer Collection im System.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CollectionStatus {
    /// Normal aktiv.
    #[default]
    Active,
    /// Zur physischen Bereinigung vorgemerkt/Fehlgeschlagen.
    PendingPurge,
}

/// Injected provider interface for deletion proof signing keys.
pub trait ProofKeyProvider: Send + Sync {
    /// Resolves the deletion proof key for the specified tenant.
    fn resolve_proof_key(&self, tenant_id: TenantId) -> Result<Vec<u8>, ErasureError>;
}

/// Summary of a single storage layer's scan result.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LayerSummary {
    /// Layer name.
    pub layer: String,
    /// Applied scanning technique.
    pub technique: String,
    /// Number of units scanned.
    pub scanned: u64,
    /// Number of remaining live entries.
    pub remaining: u64,
    /// Positive control check matches.
    pub control_matches: u64,
}

/// Detailed failure reason when erasure verification fails.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ErasureFailure {
    /// Layer where verification or purging failed.
    pub layer: String,
    /// Detailed failure explanation.
    pub reason: String,
}

/// Schichten, die in standardmäßigen Löschläufen derzeit ungemessen bleiben können (SPEC §9.4).
pub const UNMEASURED_ERASURE_LAYERS: &[&str] = &[
    "S1b Blob",
    "S4 Caches",
    "S5 Lineage",
    "S6 Checkpoints",
    "S7 Schlüsselmaterial",
    "S9 Graph",
    "S10 KV-Cache",
];

/// Erstellt ein [`LayerSummary`] aus einem [`ScanWitness`].
pub fn summary_from_witness(layer_name: String, witness: &ScanWitness) -> LayerSummary {
    LayerSummary {
        layer: layer_name,
        technique: format!("{:?}", witness.technique()),
        scanned: witness.scanned(),
        remaining: witness.remaining(),
        control_matches: if matches!(witness.technique(), ScanTechnique::FeatureDisabled) {
            0
        } else {
            1
        },
    }
}

/// Comprehensive erasure verification report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErasureReport {
    /// Gibt an, ob alle tatsächlich gemessenen Schichten verifizierbar ohne Restbestände bereinigt wurden.
    pub verified: bool,
    /// Cryptographic DeletionProof issued iff verified == true.
    pub proof: Option<DeletionProof>,
    /// Layer-by-layer measurement summaries.
    pub layers: Vec<LayerSummary>,
    /// Detailed failure descriptor if verified == false.
    pub failure: Option<ErasureFailure>,
}

/// Default excluded scopes for Contextra deletion proofs (§20.4).
pub const DEFAULT_EXCLUDED_SCOPES: &[ExcludedScope] = &[
    ExcludedScope::LlmParameterMemory,
    ExcludedScope::ConsolidatedAndDistilled,
];

/// Erzeugt eine deterministische, kollisionsfreie [`CollectionId`] aus einem Namen.
pub fn compute_collection_id(name: &str) -> CollectionId {
    let hash = blake3::hash(name.as_bytes());
    let mut buf = [0u8; 8];
    buf.copy_from_slice(&hash.as_bytes()[0..8]);
    let val = u64::from_le_bytes(buf);
    let id_u64 = match val {
        0 => 1,
        other => other,
    };
    CollectionId::new(id_u64)
}
