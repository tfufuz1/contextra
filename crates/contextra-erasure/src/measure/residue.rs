//! Port `ResidueScanner` und Restspuren-Messung (SPEC §12, WP-12).
//!
//! Stellt den synchronen Port `ResidueScanner`, zugehörige Ergebnistypen, Fehlertaxonomie
//! sowie den `ResidueMeasurementCoordinator` bereit.
//!
//! Historische Vorbilder:
//! - ca76fdac8:crates/contextra-vector/src/hnsw/residue_scan.rs:18
//! - b713f897b:crates/contextra-vector/src/hnsw/residue_scan.rs:24
//! - f228091b8:crates/contextra-vector/src/hnsw/zero_overlay.rs:27
//! - e69edff48:crates/contextra-vector/src/hnsw/witness.rs:40
//! - 48c5481e7:crates/contextra-graph/src/residue_scan.rs:17

use crate::types_local::{Clock, DocId, LayerId};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::time::Duration;

// =========================================================================
// Fehlertaxonomie
// =========================================================================

/// Spezifische Phase einer Restspuren-Messung.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResiduePhase {
    /// Initialisierung der Messung.
    Init,
    /// Basisscan vor der Löschung.
    PreEraseScan,
    /// Durchführung der Positivkontrolle.
    PositiveControl,
    /// Prüfscan nach der Löschung (Negativkontrolle).
    PostEraseScan,
    /// Berechnung des kryptographischen Zeugen-Digests.
    WitnessFinalization,
}

impl fmt::Display for ResiduePhase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Init => write!(f, "init"),
            Self::PreEraseScan => write!(f, "pre_erase_scan"),
            Self::PositiveControl => write!(f, "positive_control"),
            Self::PostEraseScan => write!(f, "post_erase_scan"),
            Self::WitnessFinalization => write!(f, "witness_finalization"),
        }
    }
}

/// Spezifische Fehlerklasse für Restspuren-Messungen.
///
/// Folgt Regel 2 (keine Stringfehler, Phasenbezug) und Regel 4 (Fail-Closed).
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ResidueMeasurementError {
    /// Positivkontrolle fehlgeschlagen: Kontrollwert wurde nicht im Index gefunden.
    #[error("positive control failed in phase {phase}: canary was not detected")]
    PositiveControlFailed {
        /// Phase, in der der Fehler auftrat.
        phase: ResiduePhase,
    },

    /// Restspuren nach der Löschung detektiert.
    #[error("residue detected in phase {phase}: {matches} matches found")]
    ResidueDetected {
        /// Phase, in der der Fehler auftrat.
        phase: ResiduePhase,
        /// Anzahl gefundener Restspuren.
        matches: u64,
    },

    /// Scanner-Fehler während der Ausführung.
    #[error("scanner error in phase {phase}: {detail}")]
    ScannerFault {
        /// Phase, in der der Fehler auftrat.
        phase: ResiduePhase,
        /// Fehlercode oder Detailkennzeichnung.
        detail: ScannerFaultKind,
    },

    /// Ungültige oder leere Positivkontroll-Vorgabe.
    #[error("invalid canary pattern in phase {phase}")]
    InvalidCanary {
        /// Phase, in der der Fehler auftrat.
        phase: ResiduePhase,
    },

    /// Arithmetischer Überlauf bei Trefferaggregation oder Zeitmessung.
    #[error("arithmetic overflow in phase {phase}")]
    ArithmeticOverflow {
        /// Phase, in der der Fehler auftrat.
        phase: ResiduePhase,
    },

    /// Inkonsistenter oder korrupter Indexzustand.
    #[error("corrupt index state detected in phase {phase}: {reason}")]
    CorruptIndexState {
        /// Phase, in der der Fehler auftrat.
        phase: ResiduePhase,
        /// Klassifikation der Korruption.
        reason: IndexCorruptionKind,
    },
}

/// Art des Fehlers im zugrundeliegenden Index-Scanner.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ScannerFaultKind {
    /// E/A- oder Medienfehler.
    IoFailure,
    /// Zugriff verweigert oder gesperrt.
    AccessDenied,
    /// Segment nicht erreichbar.
    SegmentUnavailable,
    /// Interner Fehler im Index-Adapter.
    AdapterFailure,
}

impl fmt::Display for ScannerFaultKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IoFailure => write!(f, "io_failure"),
            Self::AccessDenied => write!(f, "access_denied"),
            Self::SegmentUnavailable => write!(f, "segment_unavailable"),
            Self::AdapterFailure => write!(f, "adapter_failure"),
        }
    }
}

/// Klassifikation festgestellter Index-Korruptionen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IndexCorruptionKind {
    /// Ungültige Kantenverweise / hängende Referenzen.
    DanglingBacklink,
    /// Ungültige Header-Prüfsumme.
    BadChecksum,
    /// Unvollständige oder asynchrone Mmap-Snapshot-Reste.
    PersistedSnapshotResidue,
    /// Ungültige Slot-Markierung.
    InvalidOverlayState,
}

impl fmt::Display for IndexCorruptionKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DanglingBacklink => write!(f, "dangling_backlink"),
            Self::BadChecksum => write!(f, "bad_checksum"),
            Self::PersistedSnapshotResidue => write!(f, "persisted_snapshot_residue"),
            Self::InvalidOverlayState => write!(f, "invalid_overlay_state"),
        }
    }
}

// =========================================================================
// Domänentypen für Scananfragen und Fundstellen (ohne Rohdaten)
// =========================================================================

/// Klassifikation der Fundstelle/Speicherstruktur innerhalb der Indexschicht.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum OffsetClass {
    /// Knotenrepräsentation (z. B. Vektorslot, Knotenheader).
    VectorNode,
    /// Rückverweis- / Backlink-Tabelle.
    BacklinkEntry,
    /// Nachbarschaftsliste (Graph-Ebenen).
    NeighborList,
    /// Quantisierte Daten (z. B. Skalare U8- oder PQ-Codes).
    QuantizedData,
    /// Hilfsindizes oder Suchbaum-Knoten.
    AuxiliaryIndex,
    /// Einstiegspunkt-Referenz (Entry Point).
    EntryPoint,
}

impl fmt::Display for OffsetClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::VectorNode => write!(f, "VectorNode"),
            Self::BacklinkEntry => write!(f, "BacklinkEntry"),
            Self::NeighborList => write!(f, "NeighborList"),
            Self::QuantizedData => write!(f, "QuantizedData"),
            Self::AuxiliaryIndex => write!(f, "AuxiliaryIndex"),
            Self::EntryPoint => write!(f, "EntryPoint"),
        }
    }
}

/// Fundstelle einer Restspur im Index.
///
/// Enthält niemals vertrauliche Rohdaten oder Klartext (Datensparsamkeit).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResidueFinding {
    /// Betroffene Schicht.
    pub layer: LayerId,
    /// Strukturelle Klasse des Funds.
    pub offset_class: OffsetClass,
    /// Anzahl der Fundstellen in dieser Strukturklasse.
    pub count: u64,
}

impl ResidueFinding {
    /// Erstellt eine neue Fundstellenmeldung.
    pub const fn new(layer: LayerId, offset_class: OffsetClass, count: u64) -> Self {
        Self {
            layer,
            offset_class,
            count,
        }
    }
}

/// Gesamtergebnis eines Restspuren-Scans.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ResidueScanResult {
    /// Strukturierte Liste der Befunde.
    pub findings: Vec<ResidueFinding>,
    /// Gesamtanzahl der detektierten Treffer (saturating sum).
    pub total_matches: u64,
    /// Anzahl der untersuchten Struktureinheiten (z. B. Slots, Listen).
    pub units_checked: u64,
}

impl ResidueScanResult {
    /// Erzeugt ein leeres, sauberes Ergebnis (0 Treffer).
    pub const fn clean(units_checked: u64) -> Self {
        Self {
            findings: Vec::new(),
            total_matches: 0,
            units_checked,
        }
    }

    /// Fügt einen Fund hinzu.
    pub fn add_finding(&mut self, finding: ResidueFinding) {
        self.total_matches = self.total_matches.saturating_add(finding.count);
        self.findings.push(finding);
    }

    /// Prüft, ob keine Spuren gefunden wurden.
    pub fn is_clean(&self) -> bool {
        self.total_matches == 0 && self.findings.is_empty()
    }
}

/// Zielanfrage für einen Restspuren-Scan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResidueQuery {
    /// Dokument-ID des zu untersuchenden Datensatzes.
    pub doc_id: DocId,
    /// Fingerabdruck des Werts oder Zielobjekts (z. B. BLAKE3-Hash).
    pub target_fingerprint: [u8; 32],
}

impl ResidueQuery {
    /// Erstellt eine neue Suchanfrage.
    pub const fn new(doc_id: DocId, target_fingerprint: [u8; 32]) -> Self {
        Self {
            doc_id,
            target_fingerprint,
        }
    }
}

/// Kryptographischer Zeugen-Digest einer erfolgreichen Messung.
///
/// Kann an `contextra-erasure::report` oder `contextra-crypto` übergeben werden.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResidueWitnessDigest {
    /// Betroffene Schicht.
    pub layer: LayerId,
    /// Gesamtzeit der Verifikation.
    pub duration: Duration,
    /// Untersuchte Einheiten.
    pub units_checked: u64,
    /// BLAKE3-Digest über Prüfablauf, Zeit, Einheiten und Schicht.
    pub digest: [u8; 32],
}

impl ResidueWitnessDigest {
    /// Liefert den 32-Byte Digest als Slice.
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.digest
    }
}

// =========================================================================
// Port: ResidueScanner (synchron)
// =========================================================================

/// Synchroner Port für Index-spezifische Restspuren-Scanner (SPEC §12, WP-12).
///
/// Implementiert durch Adapter in Vektor-, Text- oder Graph-Index-Crates.
/// Arbeitet rein lesend ohne Modifikationen am Index.
pub trait ResidueScanner: Send + Sync {
    /// Liefert die Schichtkennung des Scanners.
    fn layer(&self) -> LayerId;

    /// Prüft, ob das injizierte Positivkontroll-Muster auffindbar ist.
    ///
    /// Gibt `Ok(true)` zurück, wenn das Kontrollmuster nachweisbar vorhanden ist.
    /// Gibt `Ok(false)` zurück, wenn es nicht detektiert wurde.
    fn canary_probe(&self, canary: &[u8]) -> Result<bool, ResidueMeasurementError>;

    /// Durchsucht den Index nach Spuren der spezifizierten Abfrage.
    ///
    /// Gibt ein `ResidueScanResult` zurück. Fehler führen fail-closed zum Abbruch.
    fn scan_residue(&self, query: &ResidueQuery) -> Result<ResidueScanResult, ResidueMeasurementError>;
}

// =========================================================================
// Koordinator für Restspuren-Messungen
// =========================================================================

/// Statusbericht eines vollständigen Restspuren-Messablaufs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResidueMeasurementReport {
    /// Vor-Löschung Trefferanzahl (Bestätigung der Präsenz).
    pub pre_erase_matches: u64,
    /// Nach-Löschung Trefferanzahl (muss zwingend 0 sein).
    pub post_erase_matches: u64,
    /// Nachweis der Positivkontrolle erbracht.
    pub positive_control_passed: bool,
    /// Zeugen-Digest bei erfolgreicher Null-Messung.
    pub witness: ResidueWitnessDigest,
}

/// Koordiniert den normativen Messablauf für Index-Schichten.
///
/// Ablauf:
/// 1. Positivkontrolle: Canary-Prüfung (`canary_probe`). Scheitert dies -> `PositiveControlFailed`.
/// 2. Post-Erase Scan: Restspurensuche nach Löschung (`scan_residue`).
/// 3. Validierung: Bei `total_matches > 0` -> `ResidueDetected`.
/// 4. Ausstellung: Zeugen-Digest wird nur berechnet, wenn die Positivkontrolle
///    erfolgreich war UND keine Restspuren vorhanden sind.
pub struct ResidueMeasurementCoordinator<'a> {
    scanner: &'a dyn ResidueScanner,
    clock: &'a dyn Clock,
}

impl<'a> ResidueMeasurementCoordinator<'a> {
    /// Erzeugt einen neuen Koordinator mit injiziertem Scanner und Clock.
    pub fn new(scanner: &'a dyn ResidueScanner, clock: &'a dyn Clock) -> Self {
        Self { scanner, clock }
    }

    /// Führt die Positivkontrolle auf dem Scanner aus.
    pub fn verify_positive_control(&self, canary: &[u8]) -> Result<(), ResidueMeasurementError> {
        if canary.is_empty() {
            return Err(ResidueMeasurementError::InvalidCanary {
                phase: ResiduePhase::PositiveControl,
            });
        }

        let detected = self
            .scanner
            .canary_probe(canary)
            .map_err(|e| match e {
                ResidueMeasurementError::ScannerFault { detail, .. } => {
                    ResidueMeasurementError::ScannerFault {
                        phase: ResiduePhase::PositiveControl,
                        detail,
                    }
                }
                other => other,
            })?;

        if !detected {
            return Err(ResidueMeasurementError::PositiveControlFailed {
                phase: ResiduePhase::PositiveControl,
            });
        }

        Ok(())
    }

    /// Führt die vollständige Verifikationsmessung nach physischer Löschung durch.
    ///
    /// # Parameter
    /// - `query`: Suchkriterium des gelöschten Datensatzes.
    /// - `canary`: Kontrollmuster für die Positivkontrolle.
    /// - `pre_erase_matches`: Zuvor gemessene Trefferanzahl vor der Löschung.
    pub fn measure_after_erasure(
        &self,
        query: &ResidueQuery,
        canary: &[u8],
        pre_erase_matches: u64,
    ) -> Result<ResidueMeasurementReport, ResidueMeasurementError> {
        let start_nanos = self.clock.monotonic_nanos();

        // 1. Positivkontrolle zwingend zuerst (Fail-Closed)
        self.verify_positive_control(canary)?;

        // 2. Scan nach Löschung
        let post_result = self
            .scanner
            .scan_residue(query)
            .map_err(|e| match e {
                ResidueMeasurementError::ScannerFault { detail, .. } => {
                    ResidueMeasurementError::ScannerFault {
                        phase: ResiduePhase::PostEraseScan,
                        detail,
                    }
                }
                other => other,
            })?;

        // 3. Negativkontrolle: Jede verbleibende Spur ist ein Fehler
        if post_result.total_matches > 0 || !post_result.findings.is_empty() {
            return Err(ResidueMeasurementError::ResidueDetected {
                phase: ResiduePhase::PostEraseScan,
                matches: post_result.total_matches,
            });
        }

        let elapsed_nanos = self.clock.monotonic_nanos().saturating_sub(start_nanos);
        let duration = Duration::from_nanos(elapsed_nanos);

        // 4. Zeugen-Digest deterministisch berechnen
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"ctx/residue_witness/v1");
        hasher.update(self.scanner.layer().as_bytes());
        hasher.update(&query.doc_id.inner().to_le_bytes());
        hasher.update(&query.target_fingerprint);
        hasher.update(blake3::hash(canary).as_bytes());
        hasher.update(&post_result.units_checked.to_le_bytes());
        hasher.update(&elapsed_nanos.to_le_bytes());
        let digest_bytes = *hasher.finalize().as_bytes();

        let witness = ResidueWitnessDigest {
            layer: self.scanner.layer(),
            duration,
            units_checked: post_result.units_checked,
            digest: digest_bytes,
        };

        Ok(ResidueMeasurementReport {
            pre_erase_matches,
            post_erase_matches: 0,
            positive_control_passed: true,
            witness,
        })
    }
}

// =========================================================================
// Referenz-Testdouble / Fake-Index im Speicher
// =========================================================================

/// In-Memory Testdouble für Indexstrukturen zur Verifikation der ResidueScanner-Logik.
#[derive(Debug, Clone)]
pub struct InMemoryVectorIndexStub {
    layer: LayerId,
    active_docs: std::collections::BTreeSet<u64>,
    backlinks: std::collections::BTreeMap<u64, Vec<u64>>,
    quantized_entries: std::collections::BTreeSet<u64>,
    canaries: std::collections::BTreeSet<Vec<u8>>,
    simulate_fault: Option<ScannerFaultKind>,
    simulate_corrupt: Option<IndexCorruptionKind>,
}

impl InMemoryVectorIndexStub {
    /// Erstellt einen neuen leeren Stub-Index.
    pub fn new(layer: LayerId) -> Self {
        Self {
            layer,
            active_docs: std::collections::BTreeSet::new(),
            backlinks: std::collections::BTreeMap::new(),
            quantized_entries: std::collections::BTreeSet::new(),
            canaries: std::collections::BTreeSet::new(),
            simulate_fault: None,
            simulate_corrupt: None,
        }
    }

    /// Fügt ein Dokument mit Vektorknoten und Backlinks hinzu.
    pub fn insert_document(&mut self, doc_id: DocId, backlink_targets: Vec<u64>) {
        let id = doc_id.inner();
        self.active_docs.insert(id);
        self.quantized_entries.insert(id);
        self.backlinks.insert(id, backlink_targets);
    }

    /// Registriert ein Positivkontroll-Muster (Canary).
    pub fn register_canary(&mut self, canary: Vec<u8>) {
        self.canaries.insert(canary);
    }

    /// Führt physische Bereinigung/Nullung des Dokuments durch.
    pub fn physical_erase(&mut self, doc_id: DocId) {
        let id = doc_id.inner();
        self.active_docs.remove(&id);
        self.quantized_entries.remove(&id);
        self.backlinks.remove(&id);
    }

    /// Konfiguriert die Simulation eines Scanner-Fehlers.
    pub fn set_simulate_fault(&mut self, fault: Option<ScannerFaultKind>) {
        self.simulate_fault = fault;
    }

    /// Konfiguriert die Simulation von Korruption.
    pub fn set_simulate_corrupt(&mut self, corrupt: Option<IndexCorruptionKind>) {
        self.simulate_corrupt = corrupt;
    }
}

impl ResidueScanner for InMemoryVectorIndexStub {
    fn layer(&self) -> LayerId {
        self.layer
    }

    fn canary_probe(&self, canary: &[u8]) -> Result<bool, ResidueMeasurementError> {
        if let Some(fault) = self.simulate_fault {
            return Err(ResidueMeasurementError::ScannerFault {
                phase: ResiduePhase::PositiveControl,
                detail: fault,
            });
        }
        if let Some(corrupt) = self.simulate_corrupt {
            return Err(ResidueMeasurementError::CorruptIndexState {
                phase: ResiduePhase::PositiveControl,
                reason: corrupt,
            });
        }
        Ok(self.canaries.contains(canary))
    }

    fn scan_residue(&self, query: &ResidueQuery) -> Result<ResidueScanResult, ResidueMeasurementError> {
        if let Some(fault) = self.simulate_fault {
            return Err(ResidueMeasurementError::ScannerFault {
                phase: ResiduePhase::PostEraseScan,
                detail: fault,
            });
        }
        if let Some(corrupt) = self.simulate_corrupt {
            return Err(ResidueMeasurementError::CorruptIndexState {
                phase: ResiduePhase::PostEraseScan,
                reason: corrupt,
            });
        }

        let mut result = ResidueScanResult::clean(self.active_docs.len() as u64);
        let id = query.doc_id.inner();

        if self.active_docs.contains(&id) {
            result.add_finding(ResidueFinding::new(self.layer, OffsetClass::VectorNode, 1));
        }

        if let Some(targets) = self.backlinks.get(&id) {
            if !targets.is_empty() {
                result.add_finding(ResidueFinding::new(
                    self.layer,
                    OffsetClass::BacklinkEntry,
                    targets.len() as u64,
                ));
            }
        }

        if self.quantized_entries.contains(&id) {
            result.add_finding(ResidueFinding::new(
                self.layer,
                OffsetClass::QuantizedData,
                1,
            ));
        }

        Ok(result)
    }
}
