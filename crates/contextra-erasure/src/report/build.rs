//! Zusammenstellung des signierten Löschberichts (SPEC §11.7, Anhang D, ER-27).
//!
//! Pflichtfelder (11.7):
//!   - `report_version`, `store_id`, `erasure_id`, `target_type` (nie Kennung), `commitment`
//!   - `requested_at`, `gated_at`, `key_destroyed_at`, `completed_at`
//!   - `sla_us`, `sla_violation`, `retention_code`
//!   - `field_measurement` (MK1, MK2, MK3 je Schicht) und `lab_evidence` strikt getrennt
//!   - Klassen G/K/P je `reached/not_reached` ohne Gesamtnote (12.5)
//!   - `key_coverage` je Schicht (full, unlinked, none, nicht erfasst)
//!   - `segments_rewritten`, `bytes_rewritten`
//!   - `vault_medium`, `vault_rehash_count`, `vault_old_overwritten` (D-111)
//!   - `vault_backup_scope` (D-112)
//!   - `verified_via` (D-113)
//!   - `epoch_id` (D-125)
//!   - `replaced`, `replacement_tag` (D-115)
//!   - `target = Superseded` mit `key_coverage` (D-117)
//!   - `signer_id` (D-130)
//!   - `process_hardening`
//!   - `residual_risks` (vollständiger Katalog Anhang D)
//!   - Pflichttexte: Signaturumfang (E-L11), keine Rechtsberatung, Außenaussage (6.4)

use super::canonical::to_canonical_json;
use super::risk::{evaluate_residual_risks_with_context, ResidualRiskContext, ResidualRiskEntry};
use crate::error::ErasureError;
use crate::types_local::ErasureId;
use crate::types_local::LayerWitness;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Pflichtstatus einer Nachweisklasse (G, K, P laut SPEC D-36, §12.5).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReachStatus {
    /// Erreicht
    Reached,
    /// Nicht erreicht
    NotReached,
}

impl ReachStatus {
    /// Liefert den kanonischen String "reached" oder "not_reached".
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Reached => "reached",
            Self::NotReached => "not_reached",
        }
    }
}

/// Strukturierter Zeugeneintrag für eine Schicht (SPEC §11.7, §12.2).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LayerWitnessReport {
    /// Schichtbezeichner (z. B. "doc_store", "vector_payload", "text_postings").
    pub layer: String,
    /// Messklasse ("MK1", "MK2", "MK3").
    pub class: String,
    /// Methoden-ID der Messung.
    pub method_id: String,
    /// Anzahl gescannter Dateien.
    pub scanned_files: u64,
    /// Anzahl gescannter Bytes.
    pub scanned_bytes: u64,
    /// Anzahl gefundener Rückstände (immer 0 bei gültigem Zeugen).
    pub matches: u64,
    /// Echte Positivkontrolle bestanden.
    pub control_ok: bool,
    /// Scandauer in Mikrosekunden.
    pub duration_us: u64,
    /// Hexadezimaler Hash des Prüflaufs.
    pub hash: String,
}

impl LayerWitnessReport {
    /// Erstellt einen Report-Eintrag aus einem `LayerWitness`.
    pub fn from_witness(w: &LayerWitness) -> Self {
        let layer_id = w.layer();
        let layer_id_bytes = layer_id.as_bytes();
        let layer_name = match layer_id_bytes[0] {
            1 => "doc_store".to_string(),
            2 => "vector_payload".to_string(),
            3 => "text_postings".to_string(),
            4 => "vector_edges".to_string(),
            5 => "file_structure".to_string(),
            _ => hex_encode(layer_id_bytes),
        };

        let class_str = match w.class() {
            crate::types_local::MeasureClass::Mk1 => "MK1".to_string(),
            crate::types_local::MeasureClass::Mk2 => "MK2".to_string(),
            crate::types_local::MeasureClass::Mk3 => "MK3".to_string(),
        };

        Self {
            layer: layer_name,
            class: class_str,
            method_id: format!("mk_scan_v{}", w.method_id()),
            scanned_files: w.files(),
            scanned_bytes: w.bytes(),
            matches: w.matches(),
            control_ok: true, // LayerWitness kann nur aus Armed erzeugt werden
            duration_us: w.duration().as_micros() as u64,
            hash: hex_encode(w.hash()),
        }
    }
}

/// Zeuge einer einzelnen Schicht im Bericht (Abwärtskompatibilitäts-Alias).
pub type WitnessEntry = LayerWitnessReport;

/// Feldmessungs-Abschnitt (MK1, MK2, MK3) für Abwärtskompatibilität.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FieldMeasurementSection {
    /// MK1 Status.
    pub mk1: ReachStatus,
    /// MK2 Status.
    pub mk2: ReachStatus,
    /// MK3 Status.
    pub mk3: ReachStatus,
}

/// Schlüsselabdeckung einer Schicht im Bericht.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LayerKeyCoverage {
    /// Schicht-ID (hex, kein Klartext).
    pub layer_id_hex: String,
    /// Abdeckungsgrad ("full", "unlinked", "none", "nicht erfasst").
    pub coverage: String,
}

/// Prozesshärtungsstatus (SPEC §11.7, Pflichtfeld).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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

/// Kanonischer Löschbericht (ER-27, SPEC §11.7, Anhang D).
///
/// Keine Inhalte, Schlüssel oder Klartext-Kennungen (P13).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ErasureReport {
    /// Format-Version des Berichts (immer 1 für v1.0.0).
    pub report_version: u32,
    /// Store-ID (Hexadezimal, kein Klartext).
    pub store_id: String,
    /// Lösch-ID (Hexadezimal, opak).
    pub erasure_id: String,
    /// Zieltyp ("Document", "Subject", "Collection", "Tenant", "Superseded"). Nie eine Kennung (P13).
    pub target_type: String,
    /// Kryptografisches Commitment des Ziels.
    pub commitment: String,
    /// Zeitstempel der Anfrage (Mikrosekunden UTC).
    pub requested_at: u64,
    /// Zeitstempel der Gate-Setzung (Mikrosekunden UTC).
    pub gated_at: u64,
    /// Zeitstempel der Schlüsselvernichtung (Mikrosekunden UTC).
    pub key_destroyed_at: u64,
    /// Zeitstempel des Abschlusses (Mikrosekunden UTC).
    pub completed_at: u64,
    /// Vereinbartes SLA-Fenster in Mikrosekunden.
    pub sla_us: u64,
    /// Ob das SLA verletzt wurde.
    pub sla_violation: bool,
    /// Gesetzlicher Aufbewahrungscode (SPEC §7.3).
    pub retention_code: String,
    /// Feldmessungs-Status der Schichtprüfungen (MK1, MK2, MK3).
    pub field_measurement: FieldMeasurementSection,
    /// Release-Evidenzbündel-Hash (Lab-Nachweis), strikt getrennt von field_measurement (D-72).
    pub lab_evidence: String,
    /// Klassenverband G, K, P je einzeln als reached/not_reached (SPEC §12.5, D-36).
    pub classes: BTreeMap<String, String>,
    /// Schlüsselabdeckung je Schicht (SPEC D-69).
    pub key_coverage: BTreeMap<String, String>,
    /// Anzahl neugeschriebener Segmente.
    pub segments_rewritten: u64,
    /// Anzahl neugeschriebener Bytes.
    pub bytes_rewritten: u64,
    /// Speichermedium des Tresors ("rotational", "flash", "cow", "unknown", D-111).
    pub vault_medium: String,
    /// Anzahl Rehash-Vorgänge des Tresors (D-111).
    pub vault_rehash_count: u64,
    /// Ob alte Slots beim letzten Rehash überschrieben wurden (D-111).
    pub vault_old_overwritten: bool,
    /// Sicherungsbereich des Tresors ("SeparateShortRetention", "None", "SameAsData", "Unknown", D-112).
    pub vault_backup_scope: String,
    /// Art der Verifikation ("Direct", "CacheDropped", "Cached", D-113).
    pub verified_via: String,
    /// Commitment-Epoche (D-125).
    pub epoch_id: String,
    /// Ob es sich um eine Dokumentersetzung handelte (D-115).
    pub replaced: bool,
    /// Replacement-Tag bei Ersetzung (Tag, nie Klartextinhalt, D-115).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replacement_tag: Option<String>,
    /// Signer-Identifikator (0 = K_sign abgeleitet, 1 = Betreiber, D-130).
    pub signer_id: u8,
    /// Status der Prozesshärtung (SPEC §11.7).
    pub process_hardening: String,
    /// Vollständige Liste der Restrisiken (SPEC Anhang D).
    pub residual_risks: Vec<ResidualRiskEntry>,
    /// Zeugenliste je Schicht.
    pub witnesses: Vec<LayerWitnessReport>,
    /// Pflichtsatz zum Signaturumfang (SPEC Anhang D, E-L11).
    pub signature_scope_notice: String,
    /// Pflichttext „keine Rechtsberatung“ (SPEC Anhang D).
    pub legal_notice: String,
    /// Zulässige Außenaussage nach SPEC §6.4.
    pub scope_notice: String,
    /// Optionaler Pflichttext bei Berichtigung mit Purge (SPEC Anhang D).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub superseded_purge_notice: Option<String>,
    /// Dynamische Zusatzfelder für Abwärtskompatibilität.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub fields: BTreeMap<String, String>,
}

impl ErasureReport {
    /// Abwärtskompatibler Konstruktor für bestehende Tests.
    pub fn new(schema_version: &str, timestamp_micros: u64) -> Self {
        let mut report = build_report(
            ErasureId::new([0u8; 16]),
            timestamp_micros,
            timestamp_micros,
            vec![],
            0,
            0,
            true,
        );
        report
            .fields
            .insert("schema_version".to_string(), schema_version.to_string());
        report
    }

    /// Abwärtskompatible Hilfsmethode.
    pub fn set_field(&mut self, key: &str, val: &str) {
        self.fields.insert(key.to_string(), val.to_string());
    }
}

/// Optionen für die Berichterstellung.
#[derive(Debug, Clone)]
pub struct ReportBuildOptions {
    /// Store-ID (Hex).
    pub store_id: String,
    /// Zieltyp-Name.
    pub target_type: String,
    /// Commitment-Hex.
    pub commitment: String,
    /// Anfragezeitstempel (Mikrosekunden UTC).
    pub requested_at: u64,
    /// Gate-Zeitstempel (Mikrosekunden UTC).
    pub gated_at: u64,
    /// Schlüsselvernichtungs-Zeitstempel (Mikrosekunden UTC).
    pub key_destroyed_at: u64,
    /// Abschluss-Zeitstempel (Mikrosekunden UTC).
    pub completed_at: u64,
    /// SLA in Mikrosekunden.
    pub sla_us: u64,
    /// SLA-Verletzung.
    pub sla_violation: bool,
    /// Aufbewahrungscode.
    pub retention_code: String,
    /// Release-Evidenzbündel-Hash.
    pub lab_evidence: String,
    /// Speichermedium des Tresors.
    pub vault_medium: String,
    /// Rehash-Zähler.
    pub vault_rehash_count: u64,
    /// Alte Slots überschrieben.
    pub vault_old_overwritten: bool,
    /// Backup-Scope.
    pub vault_backup_scope: String,
    /// Verifikationsmethode.
    pub verified_via: String,
    /// Epochen-ID.
    pub epoch_id: String,
    /// Ersetzt-Flag.
    pub replaced: bool,
    /// Ersatz-Tag.
    pub replacement_tag: Option<String>,
    /// Signer-ID (0 oder 1).
    pub signer_id: u8,
    /// Prozesshärtung.
    pub process_hardening: String,
    /// Kontext für Restrisiken.
    pub risk_context: ResidualRiskContext,
}

impl Default for ReportBuildOptions {
    fn default() -> Self {
        Self {
            store_id: "00000000000000000000000000000000".to_string(),
            target_type: "Document".to_string(),
            commitment: "0000000000000000000000000000000000000000000000000000000000000000"
                .to_string(),
            requested_at: 0,
            gated_at: 0,
            key_destroyed_at: 0,
            completed_at: 0,
            sla_us: 30_000_000,
            sla_violation: false,
            retention_code: "NoObligation".to_string(),
            lab_evidence: "0000000000000000000000000000000000000000000000000000000000000000"
                .to_string(),
            vault_medium: "flash".to_string(),
            vault_rehash_count: 0,
            vault_old_overwritten: true,
            vault_backup_scope: "Unknown".to_string(),
            verified_via: "Direct".to_string(),
            epoch_id: "0000000000000000".to_string(),
            replaced: false,
            replacement_tag: None,
            signer_id: 0,
            process_hardening: "Full".to_string(),
            risk_context: ResidualRiskContext::default(),
        }
    }
}

/// Erstellt einen vollständigen, normativen Löschbericht nach SPEC §11.7.
pub fn build_report_with_options(
    erasure_id: ErasureId,
    witnesses: Vec<LayerWitness>,
    segments_rewritten: u64,
    bytes_rewritten: u64,
    opts: ReportBuildOptions,
) -> ErasureReport {
    // Klassenverband G, K, P: einzeln als reached / not_reached (SPEC §12.5, D-36, T-CLASS-1)
    let mut classes = BTreeMap::new();
    let class_g = if opts.gated_at > 0 {
        ReachStatus::Reached
    } else {
        ReachStatus::NotReached
    };
    classes.insert("G".to_string(), class_g.as_str().to_string());

    let class_k = if opts.key_destroyed_at > 0 && !opts.risk_context.is_superseded_with_same_key {
        ReachStatus::Reached
    } else {
        ReachStatus::NotReached
    };
    classes.insert("K".to_string(), class_k.as_str().to_string());

    let class_p = if opts.risk_context.purge_complete {
        ReachStatus::Reached
    } else {
        ReachStatus::NotReached
    };
    classes.insert("P".to_string(), class_p.as_str().to_string());

    // key_coverage je Schicht (SPEC D-69, 12.5)
    let mut key_coverage = BTreeMap::new();
    let recorded_layers: Vec<String> = witnesses
        .iter()
        .map(|w| match w.layer().as_bytes()[0] {
            1 => "doc_store".to_string(),
            2 => "vector_payload".to_string(),
            3 => "text_postings".to_string(),
            4 => "vector_edges".to_string(),
            5 => "file_structure".to_string(),
            _ => hex_encode(w.layer().as_bytes()),
        })
        .collect();

    let standard_layers = [
        ("doc_store", "full"),
        ("vector_payload", "full"),
        ("text_postings", "unlinked"),
        ("vector_edges", "none"),
        ("file_structure", "none"),
    ];

    for (layer, expected_cov) in standard_layers {
        if recorded_layers.iter().any(|l| l == layer) || witnesses.is_empty() {
            if opts.risk_context.is_superseded_with_same_key {
                key_coverage.insert(layer.to_string(), "none".to_string());
            } else {
                key_coverage.insert(layer.to_string(), expected_cov.to_string());
            }
        } else {
            key_coverage.insert(layer.to_string(), "nicht erfasst".to_string());
        }
    }

    let mk1_status = if !witnesses.is_empty() {
        ReachStatus::Reached
    } else {
        ReachStatus::NotReached
    };
    let mk2_status = if segments_rewritten > 0 {
        ReachStatus::Reached
    } else {
        ReachStatus::NotReached
    };
    let mk3_status = if opts.risk_context.purge_complete {
        ReachStatus::Reached
    } else {
        ReachStatus::NotReached
    };
    let field_measurement = FieldMeasurementSection {
        mk1: mk1_status,
        mk2: mk2_status,
        mk3: mk3_status,
    };

    let witness_list: Vec<LayerWitnessReport> = witnesses
        .iter()
        .map(LayerWitnessReport::from_witness)
        .collect();

    let residual_risks: Vec<ResidualRiskEntry> =
        evaluate_residual_risks_with_context(&opts.risk_context)
            .into_iter()
            .map(ResidualRiskEntry::from_id)
            .collect();

    let signature_scope_notice =
        "Die Signatur beweist Integrität und Herkunft aus dieser Installation, nicht die Wahrheit des Inhalts."
            .to_string();
    let legal_notice =
        "Keine Rechtsberatung; die rechtliche Wertung obliegt dem Betreiber.".to_string();
    let scope_notice =
        "Contextra entfernt die Daten eines Löschziels aus dem Datenverzeichnis, aus den Indexen und aus den gespeicherten Zeitreise-Versionen und weist den Zustand je Schicht messtechnisch nach. Für jedes Ziel wird zusätzlich der zugehörige Schlüssel aus dem Schlüsseltresor entfernt; ob und wie das rechtlich zu werten ist, entscheidet der Betreiber. Nicht erfasst sind externe Sicherungen, Flash-Remanenz, Dateisystem-Journale, Prozessspeicher und externe Verarbeiter. Der Bericht listet diese Restrisiken."
            .to_string();

    let superseded_purge_notice = if opts.risk_context.is_superseded_with_same_key {
        Some("Die Schlüsselebene ist nicht erreicht; die Daten der Vorversion sind aus Datenverzeichnis, Indexen und Versionen entfernt.".to_string())
    } else {
        None
    };

    ErasureReport {
        report_version: 1,
        store_id: opts.store_id,
        erasure_id: hex_encode(erasure_id.as_bytes()),
        target_type: opts.target_type,
        commitment: opts.commitment,
        requested_at: opts.requested_at,
        gated_at: opts.gated_at,
        key_destroyed_at: opts.key_destroyed_at,
        completed_at: opts.completed_at,
        sla_us: opts.sla_us,
        sla_violation: opts.sla_violation,
        retention_code: opts.retention_code,
        field_measurement,
        lab_evidence: opts.lab_evidence,
        classes,
        key_coverage,
        segments_rewritten,
        bytes_rewritten,
        vault_medium: opts.vault_medium,
        vault_rehash_count: opts.vault_rehash_count,
        vault_old_overwritten: opts.vault_old_overwritten,
        vault_backup_scope: opts.vault_backup_scope,
        verified_via: opts.verified_via,
        epoch_id: opts.epoch_id,
        replaced: opts.replaced,
        replacement_tag: opts.replacement_tag,
        signer_id: opts.signer_id,
        process_hardening: opts.process_hardening,
        residual_risks,
        witnesses: witness_list,
        signature_scope_notice,
        legal_notice,
        scope_notice,
        superseded_purge_notice,
        fields: BTreeMap::new(),
    }
}

/// Erstellt einen Löschbericht mit Standardoptionen (ER-27).
pub fn build_report(
    erasure_id: ErasureId,
    gated_at: u64,
    completed_at: u64,
    witnesses: Vec<LayerWitness>,
    segments_rewritten: u64,
    bytes_rewritten: u64,
    purge_complete: bool,
) -> ErasureReport {
    let mut opts = ReportBuildOptions::default();
    opts.requested_at = gated_at.saturating_sub(100);
    opts.gated_at = gated_at;
    opts.key_destroyed_at = gated_at.saturating_add(50);
    opts.completed_at = completed_at;
    opts.risk_context.purge_complete = purge_complete;

    build_report_with_options(
        erasure_id,
        witnesses,
        segments_rewritten,
        bytes_rewritten,
        opts,
    )
}

/// Kanonisiert einen Bericht als Bytes (für die Signatur).
pub fn canonicalize_report(report: &ErasureReport) -> Result<Vec<u8>, ErasureError> {
    to_canonical_json(report)
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}
