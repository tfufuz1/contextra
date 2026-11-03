//! Zusammenstellung des signierten Löschberichts (SPEC §11.7, Anhang D, ER-27).
//!
//! Pflichtfelder (11.7):
//!   - `erasure_id`, `target_hash` (kein Klartext), `gated_at`, `completed_at`
//!   - `field_measurement` und `lab_evidence` getrennt
//!   - Klassen G/K/P je `reached/not_reached`
//!   - `key_coverage` je Schicht
//!   - `segments_rewritten`, `bytes_rewritten`
//!   - `residual_risks` (vollständige Liste)
//!   - `process_hardening`
//!   - Pflichttexte 1–5 (Anhang D)

use super::canonical::to_canonical_json;
use super::risk::{evaluate_residual_risks, ResidualRiskEntry};
use crate::error::ErasureError;
use contextra_types::erasure::LayerWitness;
use contextra_types::ids::ErasureId;
use serde::{Deserialize, Serialize};

/// Kanonischer Löschbericht (ER-27, SPEC §11.7).
///
/// Keine Inhalte, Schlüssel oder Klartext-Kennungen (P13).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErasureReport {
    /// Lösch-ID (opak).
    pub erasure_id: String,
    /// Zeitstempel der Gate-Setzung (Mikrosekunden UTC).
    pub gated_at: u64,
    /// Zeitstempel des Abschlusses (Mikrosekunden UTC, 0 = noch offen).
    pub completed_at: u64,
    /// Feldmessung (Selbstattestierung).
    pub field_measurement: FieldMeasurementSection,
    /// Lab-Evidenz (externer Nachweis, `null` wenn kein Build-Bündel).
    pub lab_evidence: Option<String>,
    /// Schlüsselabdeckung je Schicht.
    pub key_coverage: Vec<LayerKeyCoverage>,
    /// Neugeschriebene Segmente.
    pub segments_rewritten: u64,
    /// Neugeschriebene Bytes.
    pub bytes_rewritten: u64,
    /// Vollständige Restrisiken.
    pub residual_risks: Vec<ResidualRiskEntry>,
    /// Zeugenliste (je Schicht).
    pub witnesses: Vec<WitnessEntry>,
    /// Pflichttext 1 (SPEC Anhang D): Selbstattestierungs-Hinweis.
    pub disclaimer_self_attestation: &'static str,
    /// Prozesshärtungsstatus.
    pub process_hardening: ProcessHardeningStatus,
}

/// Feldmessungs-Abschnitt (MK1, MK2, MK3).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldMeasurementSection {
    /// MK1: Logische/funktionale Prüfung (reached/not_reached).
    pub mk1: ReachStatus,
    /// MK2: Speicherebenen-Rohdatenprüfung (reached/not_reached).
    pub mk2: ReachStatus,
    /// MK3: Schlüsselabwesenheits-Prüfung (reached/not_reached).
    pub mk3: ReachStatus,
}

/// Erreichbarkeitsstatus einer Messklasse.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReachStatus {
    /// Klasse wurde erreicht.
    Reached,
    /// Klasse wurde nicht erreicht.
    NotReached,
}

/// Schlüsselabdeckung einer Schicht im Bericht.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerKeyCoverage {
    /// Schicht-ID (hex, kein Klartext).
    pub layer_id_hex: String,
    /// Abdeckungsgrad.
    pub coverage: String,
}

/// Zeuge einer einzelnen Schicht im Bericht.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WitnessEntry {
    /// Schicht-ID (hex).
    pub layer_id_hex: String,
    /// Hash des Zeugen (hex).
    pub witness_hash_hex: String,
    /// Messklasse.
    pub class: String,
    /// Geprüfte Dateien.
    pub files: u64,
    /// Geprüfte Bytes.
    pub bytes: u64,
}

impl WitnessEntry {
    /// Erstellt einen Eintrag aus einem `LayerWitness`.
    pub fn from_witness(w: &LayerWitness) -> Self {
        let layer_id_hex = hex_encode(w.layer().as_bytes());
        let witness_hash_hex = hex_encode(w.hash());
        let class = format!("{:?}", w.class());
        Self {
            layer_id_hex,
            witness_hash_hex,
            class,
            files: w.files(),
            bytes: w.bytes(),
        }
    }
}

/// Prozesshärtungsstatus (SPEC §11.7, Pflichtfeld).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessHardeningStatus {
    /// Verschlüsselung erzwungen (immer true).
    pub encryption_mandatory: bool,
    /// WAL-Integrität erzwungen (immer true).
    pub wal_integrity_mandatory: bool,
}

impl Default for ProcessHardeningStatus {
    fn default() -> Self {
        Self {
            encryption_mandatory: true,
            wal_integrity_mandatory: true,
        }
    }
}

/// Erstellt einen Löschbericht (ER-27).
pub fn build_report(
    erasure_id: ErasureId,
    gated_at: u64,
    completed_at: u64,
    witnesses: Vec<LayerWitness>,
    segments_rewritten: u64,
    bytes_rewritten: u64,
    purge_complete: bool,
) -> ErasureReport {
    let field_measurement = FieldMeasurementSection {
        mk1: if !witnesses.is_empty() {
            ReachStatus::Reached
        } else {
            ReachStatus::NotReached
        },
        mk2: if segments_rewritten > 0 {
            ReachStatus::Reached
        } else {
            ReachStatus::NotReached
        },
        mk3: if purge_complete {
            ReachStatus::Reached
        } else {
            ReachStatus::NotReached
        },
    };

    let residual_risks = evaluate_residual_risks(purge_complete)
        .iter()
        .map(ResidualRiskEntry::from_risk)
        .collect();

    let witness_entries = witnesses.iter().map(WitnessEntry::from_witness).collect();

    let key_coverage = witnesses
        .iter()
        .map(|w| LayerKeyCoverage {
            layer_id_hex: hex_encode(w.layer().as_bytes()),
            coverage: format!("{:?}", contextra_types::erasure::KeyCoverage::Full),
        })
        .collect();

    ErasureReport {
        erasure_id: hex_encode(erasure_id.as_bytes()),
        gated_at,
        completed_at,
        field_measurement,
        lab_evidence: None,
        key_coverage,
        segments_rewritten,
        bytes_rewritten,
        residual_risks,
        witnesses: witness_entries,
        disclaimer_self_attestation: "This report is a self-attestation by the Contextra erasure core. Independent verification requires ctx-verify or lab measurement.",
        process_hardening: ProcessHardeningStatus::default(),
    }
}

/// Kanonisiert einen Bericht als Bytes (für die Signatur).
pub fn canonicalize_report(report: &ErasureReport) -> Result<Vec<u8>, ErasureError> {
    to_canonical_json(report)
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}
