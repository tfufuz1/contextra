#![allow(deprecated)]
//! Restmessung und Kontexttypen für physische Löschnachweise (§4.2, §20.5).
//!
//! # Zweck & Architektur-Rolle: Typen-Fundament (contracts)
//!
//! Für gesetzliche und regulatorische Nachweise muss nachgewiesen werden,
//! dass gelöschte Daten weder im Speicher, noch im WAL oder in Vektor-/Text-Indizes
//! als Geisterdaten verbleiben.
//!
//! Dieses Modul definiert:
//! - `ScanMethod`: Angewandte Methodik bei der Restprüfung der Speicherschichten (deprecated).
//! - `RequestContext`: Autorisierungskontext mit `TenantId` und `TxId`.
//! - `SagaRecord`: Transaktions-Hash-Zusammenfassung für Lösch-Sagas (deprecated).
//!
//! # Deprecated
//! Löschdomänen-Typen (`ScanMethod`, `SagaRecord`) sind nach `acta-erasure` migriert.

// FILE-CONTEXT
// ZWECK: Typsichere Restmessung und Basistypen für physische Löschnachweise (§4.2, §20.5, INV-WITNESS-MINT-1).
// INVARIANTEN: LayerWitness liegt ausschließlich in acta-ports::scanner.
// HOTSPOTS: [1-40]

#![forbid(unsafe_code)]
//! Residual measurement and context types (§4.2, §20.5).
//!
//! # Architektur-Rolle
//! Typen-Fundament (Kern). Keine I/O- oder Asynchron-Abhängigkeiten.
//!
//! # Zweck
//! Stellt Grundtypen für Restmessungen (Residuen) und Anforderungskontexte für
//! Löschprozesse zur Verfügung.

use serde::{Deserialize, Serialize};

// ANCHOR[acta-types:WITNESS-001]
/// Scanner methodology applied during storage residue verification.
// MIGRATION(S-70): ScanMethod gehört nach acta-erasure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
// ANCHOR[acta-types:WIT-001]
#[deprecated(
    since = "0.1.0",
    note = "Löschtypen gehören in die Erasure-Familie (acta-erasure-spi per D-E1)"
)]
pub enum ScanMethod {
    /// Direct structural scan of storage files, indices, or tables.
    Structural,
    /// Layer was not measured during this execution with specified reason.
    NotMeasured(String),
    /// Layer measurement is disabled due to feature configuration.
    FeatureDisabled,
}

// ANCHOR[acta-types:WITNESS-002]
/// Request context containing tenant and transaction identity for deletion operations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
// ANCHOR[acta-types:WIT-002]
pub struct RequestContext {
    /// Tenant identifier for authorization and scope isolation.
    pub tenant_id: crate::TenantId,
    /// Transaction ID allocated for the operation.
    pub tx_id: crate::TxId,
}

impl RequestContext {
    // ANCHOR[acta-types:WITNESS-003]
    /// Creates a new `RequestContext`.
    ///
    /// # Parameter
    /// - `tenant_id`: Mandantenidentifikator.
    /// - `tx_id`: Zugeordnete Transaktions-ID.
    pub fn new(tenant_id: crate::TenantId, tx_id: crate::TxId) -> Self {
        Self { tenant_id, tx_id }
    }
}

// ANCHOR[acta-types:WITNESS-004]
/// Saga record containing deletion tracking state and digests.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
// ANCHOR[acta-types:WIT-003]
#[deprecated(
    since = "0.1.0",
    note = "Löschtypen gehören in die Erasure-Familie (acta-erasure-spi per D-E1)"
)]
pub struct SagaRecord {
    /// BLAKE3 hash of deleted keys.
    pub deleted_keys_hash: [u8; 32],
}

impl SagaRecord {
    // ANCHOR[acta-types:WITNESS-005]
    /// Creates a new `SagaRecord`.
    ///
    /// # Parameter
    /// - `deleted_keys_hash`: BLAKE3-Hash der zu löschenden Schlüssel.
    pub fn new(deleted_keys_hash: [u8; 32]) -> Self {
        Self { deleted_keys_hash }
    }
}
