//! # Checkpoint-Manifest und Integritätsprüfung
//!
//! Dieses Modul implementiert das kryptografisch verifizierte Manifest eines Checkpoints.
//!
//! ## Ring & Rolle
//! Gehört zu **Ring 1 (Persistenz)**. Schützt Checkpoint-Metadaten und Komponentenlisten vor
//! versehentlicher Beschädigung oder unbemerkter Manipulation (Tampering).
//!
//! ## Hauptkomponenten
//! - [`CheckpointManifest`]: Struktur bestehend aus Metadaten, Komponentenliste und BLAKE3-Prüfsumme.

use crate::meta::{validate_identifier, CheckpointMeta};
use contextra_types::{ContextraError, Result};
use serde::{Deserialize, Serialize};

/// AI-TAG\[PANIC-SAFETY\]\[CRITICAL\] RESOLVED: AGT-CKPT-f3a1b2c4 (TS:2026-08-29T08:06:29Z) (SESSION:14348074)
/// Fault-Injection-Tests in
/// tests/manifest_fault_injection.rs beweisen atomare Schreib-Semantik
/// und Tamper-Erkennung via Blake3-Checksum.
// ANCHOR[contextra-checkpoint:MAN-001]
/// Kryptografisch gesichertes Manifest eines gespeicherten Checkpoints.
///
/// # Zweck
/// Bindet Metadaten (`CheckpointMeta`) und zugehörige Komponenten-Dateien (z. B. SSTable-Listen)
/// an eine BLAKE3-Prüfsumme, um Teil-Schreibfehler oder Manipulationen sofort zu erkennen.
///
/// # Felder
/// - `meta`: Die logischen Metadaten des Checkpoints.
/// - `components`: Liste der enthaltenen Komponenten-/Dateinamen.
/// - `checksum`: Hexadezimaler BLAKE3-Hash über die serialisierte Kombination aus `meta` und `components`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CheckpointManifest {
    pub meta: CheckpointMeta,
    pub components: Vec<String>,
    pub checksum: String,
}

impl CheckpointManifest {
    // ANCHOR[contextra-checkpoint:MAN-002]
    /// Erzeugt ein neues [`CheckpointManifest`] und berechnet die BLAKE3-Prüfsumme.
    ///
    /// # Vorbedingungen
    /// - `meta.name` und `meta.collection_id` müssen gültige Bezeichner sein (nicht leer, <= 256 Zeichen).
    /// - Kein Komponentenname darf nach Trimming leer sein.
    ///
    /// # Ablauf von `new`:
    /// 1. Bezeichner `name` und `collection_id` über `validate_identifier` validieren.
    /// 2. Alle Komponenten durchlaufen; bei leeren Einträgen Fehler `InvalidInput` zurückgeben.
    /// 3. Tupel `(&meta, &components)` deterministisch als JSON serialisieren.
    /// 4. BLAKE3-Hash über den Bytestrom berechnen und als Hex-String speichern.
    /// 5. [`CheckpointManifest`]-Instanz erzeugen und zurückgeben.
    ///
    /// # Fehler
    /// Gibt [`ContextraError::InvalidInput`] bei unzulässigen Namen/Komponenten zurück,
    /// oder [`ContextraError::Serialization`] bei JSON-Fehlern.
    pub fn new(meta: CheckpointMeta, components: Vec<String>) -> Result<Self> {
        validate_identifier("Checkpoint name", &meta.name)?;
        validate_identifier("Collection ID", &meta.collection_id)?;
        for comp in &components {
            if comp.trim().is_empty() {
                return Err(ContextraError::InvalidInput(
                    "Checkpoint component name cannot be empty".to_string(),
                ));
            }
        }
        // NOTE: Serialisierung von (&meta, &components) sichert die exakte Bindung beider Felder
        let payload = serde_json::to_vec(&(&meta, &components))
            .map_err(|e| ContextraError::Serialization(e.to_string()))?;
        let checksum = blake3::hash(&payload).to_hex().to_string();
        Ok(Self {
            meta,
            components,
            checksum,
        })
    }

    // ANCHOR[contextra-checkpoint:MAN-003]
    /// Verifiziert die Integrität des Manifests anhand der gespeicherten Prüfsumme.
    ///
    /// # Ablauf von `verify`:
    /// 1. Tupel `(&self.meta, &self.components)` neu nach JSON serialisieren.
    /// 2. BLAKE3-Hash über das Payload berechnen.
    /// 3. Berechneten Hash mit `self.checksum` abgleichen.
    /// 4. Bei Diskrepanz Fehler `Serialization` zurückliefern (Fail-Closed).
    ///
    /// # Fehler
    /// Gibt [`ContextraError::Serialization`] zurück, falls die Prüfsumme nicht übereinstimmt.
    pub fn verify(&self) -> Result<()> {
        let payload = serde_json::to_vec(&(&self.meta, &self.components))
            .map_err(|e| ContextraError::Serialization(e.to_string()))?;
        let expected = blake3::hash(&payload).to_hex().to_string();
        // NOTE: Bei Mismatch strikt abbrechen, um manipulierte oder korrumpierte Daten abzuweisen
        if self.checksum != expected {
            return Err(ContextraError::Serialization(format!(
                "Checkpoint manifest checksum mismatch for '{}': expected {}, got {}",
                self.meta.name, expected, self.checksum
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;
    use contextra_types::TxId;

    // Prüft, dass eine Komponente mit reinem Whitespace bei der Manifest-Erstellung abgelehnt wird.
    #[test]
    fn test_manifest_validation_blank_component() {
        let meta = CheckpointMeta {
            name: "cp1".to_string(),
            collection_id: "col1".to_string(),
            seq_no: 1,
            tx_id: TxId::new(1),
            metadata: serde_json::json!({}),
            created_at: 100,
        };

        let res = CheckpointManifest::new(meta, vec!["   ".to_string()]);
        assert!(matches!(res, Err(ContextraError::InvalidInput(_))));
    }

    // Prüft, dass Whitespace-Komponenten abgewiesen werden (Namenskonvention CASE).
    #[allow(non_snake_case)]
    #[test]
    fn checkpoint_manifest_CASE_whitespace_component_rejected() {
        let meta = CheckpointMeta {
            name: "cp_ws".to_string(),
            collection_id: "col_ws".to_string(),
            seq_no: 1,
            tx_id: TxId::new(10),
            metadata: serde_json::json!({}),
            created_at: 100,
        };
        let res = CheckpointManifest::new(meta, vec!["   ".to_string()]);
        assert!(matches!(res, Err(ContextraError::InvalidInput(_))));
    }

    // Prüft, dass die Verifikation fehlschlägt, wenn Komponenten nachträglich manipuliert wurden.
    #[allow(non_snake_case)]
    #[test]
    fn checkpoint_manifest_CASE_tampered_manifest_fails_verify() {
        let meta = CheckpointMeta {
            name: "cp_tamper".to_string(),
            collection_id: "col_tamper".to_string(),
            seq_no: 5,
            tx_id: TxId::new(50),
            metadata: serde_json::json!({"version": 1}),
            created_at: 500,
        };
        let mut manifest = CheckpointManifest::new(meta, vec!["comp1".to_string()])
            .expect("// expect #[cfg(test)]");
        manifest.components.push("tampered_comp".to_string());
        let res = manifest.verify();
        assert!(matches!(res, Err(ContextraError::Serialization(_))));
    }

    // Prüft, dass ein ungültiger Checkpoint-Name in CheckpointMeta bei der Manifest-Erstellung abgelehnt wird.
    #[test]
    fn test_manifest_creation_invalid_meta_name() {
        let meta = CheckpointMeta {
            name: "   ".to_string(),
            collection_id: "col_valid".to_string(),
            seq_no: 1,
            tx_id: TxId::new(10),
            metadata: serde_json::json!({}),
            created_at: 100,
        };
        let res = CheckpointManifest::new(meta, vec!["comp1".to_string()]);
        assert!(matches!(res, Err(ContextraError::InvalidInput(_))));
    }

    // Property-basierter Test für die Konsistenz und Verifikation von Manifesten über diverse Eingabekombinationen.
    proptest::proptest! {
        #[test]
        fn prop_manifest_roundtrip(
            name: String,
            col: String,
            seq_no: u64,
            tx: u64,
            created_at: u64,
        ) {
            if name.trim().is_empty()
                || col.trim().is_empty()
                || name.len() > 256
                || col.len() > 256
            {
                // Expected validation failure for invalid boundaries
                let meta = CheckpointMeta {
                    name,
                    collection_id: col,
                    seq_no,
                    tx_id: TxId::new(tx),
                    metadata: serde_json::json!({}),
                    created_at,
                };
                proptest::prop_assert!(CheckpointManifest::new(meta, vec!["valid_comp".to_string()]).is_err());
            } else {
                let meta = CheckpointMeta {
                    name: name.clone(),
                    collection_id: col.clone(),
                    seq_no,
                    tx_id: TxId::new(tx),
                    metadata: serde_json::json!({}),
                    created_at,
                };
                let manifest = CheckpointManifest::new(meta, vec!["comp_a".to_string(), "comp_b".to_string()]);
                proptest::prop_assert!(manifest.is_ok());
                let manifest = manifest.expect("// expect #[cfg(test)]");
                proptest::prop_assert!(manifest.verify().is_ok());
                proptest::prop_assert_eq!(manifest.meta.name, name);
                proptest::prop_assert_eq!(manifest.meta.collection_id, col);
                proptest::prop_assert_eq!(manifest.meta.seq_no, seq_no);
                proptest::prop_assert_eq!(manifest.meta.tx_id, TxId::new(tx));
                proptest::prop_assert_eq!(manifest.meta.created_at, created_at);
            }
        }
    }
}
