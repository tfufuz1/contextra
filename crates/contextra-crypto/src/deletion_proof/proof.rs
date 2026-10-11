//! Core DeletionProof struct definition and common helpers.

use super::types::{DeletionLayer, DeletionScope, ExcludedScope, GraphRepairAttestation};
use contextra_types::{TenantId, TxId};
use serde::{Deserialize, Serialize};

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
    /// Integritätswarnung für Legacy-Proofs (Version 1).
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub integrity_warning: Option<String>,
}

pub(super) const fn default_signature_version() -> u8 {
    1
}

impl DeletionProof {
    /// Gibt die Tenant-ID aus dem Scope zurück.
    pub fn tenant_id(&self) -> TenantId {
        match &self.scope {
            DeletionScope::Document { tenant_id, .. } => *tenant_id,
            DeletionScope::Collection { tenant_id, .. } => *tenant_id,
            DeletionScope::Tenant { tenant_id } => *tenant_id,
        }
    }
}
