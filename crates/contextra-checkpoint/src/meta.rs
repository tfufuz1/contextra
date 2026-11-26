//! # Checkpoint-Metadaten und Zustandsmodelle
//!
//! Dieses Modul definiert die Datenstrukturen für benannte Checkpoints und transaktionale Agenten-Zustände.
//!
//! ## Ring & Rolle
//! Gehört zu **Ring 1 (Persistenz)**. Definiert die serialisierbaren Metadatenstrukturen für persistente Checkpoints
//! (`CheckpointMeta`) sowie transaktionale Zwischenzustände (`StateCheckpoint`).
//!
//! ## Hauptkomponenten
//! - [`CheckpointMeta`]: Enthält Name, Mandanten-Collection-ID, LSM-Sequenznummer, Transaktions-ID und Freiform-Metadaten.
//! - [`StateCheckpoint`]: Minimaler Transaktions-Snapshot mit Zeitstempel und optionalem Namespace.
//! - [`validate_identifier`]: Validierungsfunktion für sichere String-Bezeichner.

use contextra_types::{ContextraError, Result, TxId, WorkflowState};
use serde::{Deserialize, Serialize};

// RESOLVED: AGT-CKPT-001 — UTF-8 char counting used for 256 char limit (TS: 2026-09-01T23:07:05Z) (SESSION: 358e3b0a)
// ANCHOR[contextra-checkpoint:VAL-001]
/// Validiert Bezeichner-Strings (z. B. Checkpoint-Namen, Collection-IDs) gegen leere Eingaben und Längenbeschränkungen.
///
/// # Zweck
/// Verhindert ungültige oder schädliche Pfade/Schlüssel in der Speicher-Engine und garantiert
/// wohldefinierte Namensräume.
///
/// # Ablauf von `validate_identifier`:
/// 1. Prüfen, ob der getrimmte Wert leer oder reines Whitespace ist -> Fehler `InvalidInput`.
/// 2. Zählen der Unicode-Zeichen (UTF-8 Scalar Values).
/// 3. Prüfen, ob die Zeichenanzahl maximal 256 beträgt -> bei Überschreitung Fehler `InvalidInput`.
///
/// # Fehler
/// Gibt [`ContextraError::InvalidInput`] zurück, wenn der String leer ist oder mehr als 256 Zeichen enthält.
pub(crate) fn validate_identifier(field_name: &str, value: &str) -> Result<()> {
    // NOTE: Whitespace-Prüfung verhindert versehentliche Leerzeichen als Bezeichner
    if value.trim().is_empty() {
        return Err(ContextraError::InvalidInput(format!(
            "{field_name} cannot be empty or whitespace-only"
        )));
    }
    let char_count = value.chars().count();
    // NOTE: 256 Zeichen Begrenzung schützt vor DoS und Pufferüberläufen in Speicherformaten
    if char_count > 256 {
        return Err(ContextraError::InvalidInput(format!(
            "{field_name} exceeds maximum length of 256 characters (got {char_count})"
        )));
    }
    Ok(())
}

// ANCHOR[contextra-checkpoint:META-001]
/// Metadaten für einen persistenten, benannten Checkpoint.
///
/// # Zweck
/// Speichert alle Identifikations-, Versions- und Audit-Informationen zu einem Snapshot
/// im LSM-Tree und Vektor-Index.
///
/// # Felder
/// - `name`: Eindeutiger Name des Checkpoints (z. B. "pre_migration").
/// - `collection_id`: Eindeutige Kennung der Collection bzw. des Mandanten.
/// - `seq_no`: LSM-Sequenznummer (MVCC-Schnittlinie für Lese-Isolation).
/// - `tx_id`: Transaktions-ID, die den Zustand erzeugt hat.
/// - `metadata`: JSON-Objekt für anwendungsspezifische Kontextdaten (z. B. Modell-Version, Prompt-ID).
/// - `created_at`: Unix-Zeitstempel der Erstellung in Millisekunden.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CheckpointMeta {
    pub name: String,
    pub collection_id: String,
    pub seq_no: u64,
    pub tx_id: TxId,
    pub metadata: serde_json::Value,
    pub created_at: u64,
}

impl CheckpointMeta {
    /// Konvertiert die Metadaten in einen [`WorkflowState`].
    ///
    /// # Kompatibilität
    /// Behält `&self` für Rückwärtskompatibilität mit externen Aufrufern bei (ADR-011).
    #[allow(clippy::wrong_self_convention)]
    pub fn into_workflow_state(&self) -> WorkflowState {
        self.to_workflow_state()
    }

    // ANCHOR[contextra-checkpoint:META-002]
    /// Konvertiert die Metadaten in einen [`WorkflowState`].
    ///
    /// # Ablauf:
    /// 1. Transaktions-ID aus `tx_id` übernehmen.
    /// 2. Deterministic Hash aus der Sequenznummer mit BLAKE3 bilden (`seq-<seq_no>`).
    /// 3. [`WorkflowState`] mit Transaktion und Graph-Hash zurückliefern.
    pub fn to_workflow_state(&self) -> WorkflowState {
        // NOTE: Der Hash wird deterministisch aus der Sequenznummer erzeugt, um Graph-Zustände abzugleichen
        WorkflowState {
            tx: self.tx_id,
            graph_hash: *blake3::hash(format!("seq-{}", self.seq_no).as_bytes()).as_bytes(),
        }
    }
}

// ANCHOR[contextra-checkpoint:STATE-001]
/// Point-in-Time Checkpoint für einen Agenten-Schritt oder eine Transaktionsgrenze.
///
/// # Zweck
/// Repräsentiert einen leichten Transaktions-Zwischenstand zur Laufzeit, z. B. während
/// mehrstufiger Agenten-Schleifen.
///
/// # Felder
/// - `tx_id`: Transaktions-ID des Zwischenschritts.
/// - `timestamp_ms`: Monotoner Zeitstempel in Millisekunden (über injected Clock-Port generiert).
/// - `namespace`: Optionaler logischer Namensraum zur Isolation von Workflows.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StateCheckpoint {
    pub tx_id: TxId,
    pub timestamp_ms: u64,
    #[serde(default)]
    pub namespace: Option<String>,
}

#[cfg(test)]
#[allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]
mod tests {
    use super::*;

    // Prüft die fehlerfreie JSON-Serialisierung und -Deserialisierung von CheckpointMeta (Roundtrip).
    #[allow(non_snake_case)]
    #[test]
    fn checkpoint_meta_CASE_serialization_roundtrip() {
        let meta = CheckpointMeta {
            name: "cp_test".to_string(),
            collection_id: "col_test".to_string(),
            seq_no: 99,
            tx_id: TxId::new(1001),
            metadata: serde_json::json!({"step": 42, "env": "prod"}),
            created_at: 1690000000,
        };

        let json = serde_json::to_string(&meta).expect("// expect #[cfg(test)]");
        let deserialized: CheckpointMeta =
            serde_json::from_str(&json).expect("// expect #[cfg(test)]");

        // Independent evaluation without referencing meta in comparison construction
        assert_eq!(deserialized.name, "cp_test");
        assert_eq!(deserialized.collection_id, "col_test");
        assert_eq!(deserialized.seq_no, 99);
        assert_eq!(deserialized.tx_id, TxId::new(1001));
        assert_eq!(deserialized.metadata["step"], 42);
        assert_eq!(deserialized.created_at, 1690000000);
    }

    // Prüft die fehlerfreie JSON-Serialisierung und -Deserialisierung von StateCheckpoint mit Namespace.
    #[allow(non_snake_case)]
    #[test]
    fn state_checkpoint_CASE_serialization_roundtrip() {
        let cp = StateCheckpoint {
            tx_id: TxId::new(888),
            timestamp_ms: 1700000000123,
            namespace: Some("test_ns".to_string()),
        };

        let json = serde_json::to_string(&cp).expect("// expect #[cfg(test)]");
        let deserialized: StateCheckpoint =
            serde_json::from_str(&json).expect("// expect #[cfg(test)]");

        assert_eq!(deserialized.tx_id, TxId::new(888));
        assert_eq!(deserialized.timestamp_ms, 1700000000123);
        assert_eq!(deserialized.namespace, Some("test_ns".to_string()));
    }

    // Prüft die Rückwärtskompatibilität beim Deserialisieren alter JSON-Zustände ohne Namespace-Feld.
    #[test]
    fn state_checkpoint_deserializes_legacy_json_without_namespace() {
        let legacy_json = r#"{"tx_id": 999, "timestamp_ms": 1700000000000}"#;
        let deserialized: StateCheckpoint = serde_json::from_str(legacy_json)
            .expect("Legacy JSON without namespace must deserialize");

        assert_eq!(deserialized.tx_id, TxId::new(999));
        assert_eq!(deserialized.timestamp_ms, 1700000000000);
        assert_eq!(deserialized.namespace, None);
    }

    // Prüft die korrekte Ableitung eines WorkflowState aus CheckpointMeta inklusive deterministischem Graph-Hash.
    #[allow(non_snake_case)]
    #[test]
    fn into_workflow_state_CASE_valid_conversion() {
        let meta = CheckpointMeta {
            name: "wf_cp".to_string(),
            collection_id: "wf_col".to_string(),
            seq_no: 15,
            tx_id: TxId::new(2026),
            metadata: serde_json::json!({"agent_phase": "reasoning"}),
            created_at: 5000,
        };

        let state = meta.into_workflow_state();

        // Independent expected value assertions
        assert_eq!(state.tx, TxId::new(2026));
        assert_ne!(state.graph_hash, [0u8; 32]);
    }

    // Prüft Grenzwerte für Identifier-Längen (exakt 256 Zeichen erlaubt, 257 Zeichen abgewiesen).
    #[test]
    fn test_validate_identifier_boundary_oversized_257() {
        let name_256 = "a".repeat(256);
        assert!(validate_identifier("field", &name_256).is_ok());

        let name_257 = "a".repeat(257);
        let res = validate_identifier("field", &name_257);
        assert!(matches!(res, Err(ContextraError::InvalidInput(_))));
    }

    // Prüft, dass leere Strings oder reine Whitespace-Strings als Identifier abgewiesen werden.
    #[test]
    fn test_validate_identifier_empty_or_whitespace() {
        let res_empty = validate_identifier("field", "");
        assert!(matches!(res_empty, Err(ContextraError::InvalidInput(_))));

        let res_ws = validate_identifier("field", "   \t\n  ");
        assert!(matches!(res_ws, Err(ContextraError::InvalidInput(_))));
    }
}
