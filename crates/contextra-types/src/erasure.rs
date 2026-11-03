//! Löschtypen und typisierter Zeuge (SPEC 7.3, 7.4, 8.1, 11.2, 11.3, 12.1, 12.2, Anhang D, D-69, D-72, I-19, L-5, TY-09, TY-10, TY-13).

use core::fmt;
use core::time::Duration;
use serde::{Deserialize, Serialize};

use crate::ids::{ErasureId, Timestamp};
use crate::refs::{CollectionRef, DocRef, OperatorRef, SubjectRef, TenantRef};

/// Löschziel (SPEC 7.3).
#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum EraseTarget {
    /// Einzelnes Dokument.
    Document {
        /// Mandanten-Referenz.
        tenant: TenantRef,
        /// Collection-Referenz.
        collection: CollectionRef,
        /// Dokument-Referenz (inklusive Subject).
        doc: DocRef,
    },
    /// Betroffener über alle Dokumente einer oder mehrerer Collections (D-14).
    Subject {
        /// Mandanten-Referenz.
        tenant: TenantRef,
        /// Betroffenen-Referenz.
        subject: SubjectRef,
    },
    /// Gesamte Collection.
    Collection {
        /// Mandanten-Referenz.
        tenant: TenantRef,
        /// Collection-Referenz.
        collection: CollectionRef,
    },
    /// Mandanten-Offboarding (D-38).
    Tenant {
        /// Mandanten-Referenz.
        tenant: TenantRef,
    },
}

/// Zeitliches Verhalten der Löschung (SPEC 7.3).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum Timing {
    /// Sofortige Löschung im Vordergrund.
    Now,
    /// Gebündelte Löschung im Batch-Fenster.
    Batched,
}

/// Löschmodus (SPEC 7.3).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum EraseMode {
    /// Vollständige physische Löschung und Schlüsselvernichtung.
    Erase,
    /// Einschränkung der Verarbeitung (Art. 18 DSGVO).
    Restrict,
}

/// Abschließender Aufbewahrungs-Code (SPEC 7.3, E-L12, TY-13).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum RetentionCode {
    /// Es besteht keine Aufbewahrungspflicht.
    NoObligation,
    /// Aufbewahrungspflicht bestand und ist abgelaufen.
    ObligationExpired,
    /// Aufbewahrungspflicht besteht aktiv.
    ObligationActive,
    /// Rechtsansprüche sind anhängig.
    LegalClaimsPending,
    /// Betroffene Person hat Einschränkung verlangt.
    DataSubjectRequestedRestriction,
    /// Betreiber trägt Verantwortung per Direktive / Anordnung.
    OperatorOverride,
}

/// Pflicht-Attestierung zur Aufbewahrung bei jedem Löschauftrag (SPEC 7.3, D-15).
#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct RetentionAttestation {
    /// Aufbewahrungscode.
    pub code: RetentionCode,
    /// Opake Betreiberkennung (nicht im Klartext im Bericht).
    pub attested_by: OperatorRef,
    /// Zeitstempel der Attestierung.
    pub attested_at: Timestamp,
}

/// Löschauftrag (SPEC 7.3).
#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct EraseRequest {
    /// Löschziel.
    pub target: EraseTarget,
    /// Timing (sofort oder gebündelt).
    pub timing: Timing,
    /// Aufbewahrungs-Attestierung.
    pub retention: RetentionAttestation,
    /// Modus (Löschen oder Sperren).
    pub mode: EraseMode,
}

/// Quittung eines Löschauftrags (SPEC 7.4).
#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct EraseReceipt {
    /// Eindeutige Kennung des Löschvorgangs.
    pub id: ErasureId,
    /// Status der Löschung.
    pub status: ErasureStatus,
    /// Zeitstempel der Sperrung im Gate.
    pub gated_at: Timestamp,
}

/// Schicht-ID einer beteiligten Speicherschicht (SPEC 8.2).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct LayerId(pub [u8; 16]);

impl LayerId {
    /// Erzeugt eine neue `LayerId`.
    pub const fn new(bytes: [u8; 16]) -> Self {
        Self(bytes)
    }

    /// Liefert die rohen Bytes der Schicht-ID.
    pub const fn as_bytes(&self) -> &[u8; 16] {
        &self.0
    }
}

impl fmt::Debug for LayerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "LayerId({:02x}{:02x}{:02x}{:02x}..)",
            self.0[0], self.0[1], self.0[2], self.0[3]
        )
    }
}

/// Begründung für Verzögerungen bei der Löschabwicklung (SPEC 11.2).
#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum PendingReason {
    /// I/O-Fehler bei der Schichtbearbeitung.
    Io,
    /// SLA-Zeitfenster überschritten.
    SlaExceeded,
    /// Eine Schicht ist fehlgeschlagen.
    LayerFailed(LayerId),
    /// Nachweis/Zeuge fehlt noch.
    WitnessMissing,
    /// Plattenreserve unterschritten.
    DiskReserve,
}

/// Begründung für die Ablehnung eines Löschauftrags (SPEC 7.3, 7.4).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum RefusalReason {
    /// Gesetzliche Aufbewahrungspflicht steht der Löschung entgegen.
    RetentionObligation,
    /// Unzulässige Parameterkombination.
    InvalidParameters,
}

/// Status der Lösch-Saga (SPEC 7.4, 11.2).
#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum ErasureStatus {
    /// Auftrag empfangen und validiert.
    Requested,
    /// Im Gate persistiert, Lesepfade gesperrt.
    Gated,
    /// Schlüssel in Keyslots dauerhaft vernichtet.
    KeyDestroyed,
    /// Physisches Umschreiben und Bereinigen läuft.
    Purging,
    /// Unabhängige Messung läuft.
    Verifying,
    /// Löschung vollständig nachgewiesen und abgeschlossen.
    Completed,
    /// Verzögert mit sichtbarem Grund (nie still).
    Pending {
        /// Grund für den Schwebezustand.
        reason: PendingReason,
    },
    /// Abgelehnt vor Gate.
    Refused(RefusalReason),
}

/// Reichweite der Schlüsselabdeckung einer Schicht (SPEC 8.2, D-69).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum KeyCoverage {
    /// Vollständige kryptografische Unlesbarkeit aller Records/Payloads.
    Full,
    /// Struktur sichtbar, aber Verknüpfung unlesbar gemacht.
    Unlinked,
    /// Keine Schlüsselvernichtung auf dieser Schichtebene.
    None,
}

/// Gate-Zustandsvariante (SPEC 11.3).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum GateVariant {
    /// Klasse P: Physisch gelöscht und gesperrt.
    Erased,
    /// Einschränkung der Verarbeitung aktiv.
    Restricted,
}

/// Messklasse für unabhängige Löschkontrollen (SPEC 12.1).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum MeasureClass {
    /// MK1: Logische/funktionale Prüfung.
    Mk1,
    /// MK2: Speicherebenen-Rohdatenprüfung.
    Mk2,
    /// MK3: Physische Medieninspektion.
    Mk3,
}

/// Risikobedingung zur Evaluierung von Restrisiken (SPEC Anhang D, AR-4).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum RiskCondition {
    /// Bedingung trifft immer zu.
    Always,
    /// Trifft zu bei CoW-Dateisystemen oder Journaling.
    CowOrJournaling,
    /// Trifft zu bis zum nächsten Kompaktions- und Purge-Durchlauf.
    UntilPurgeComplete,
}

/// Abschließender Katalog von 15 Restrisiken (SPEC Anhang D).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum ResidualRisk {
    /// Flash-Remanenz auf FTL-Ebene.
    FlashRemanence,
    /// CoW-Dateisysteme oder Dateisystemjournale.
    FsJournalOrCow,
    /// Reste verpackter Schlüsselkopien.
    WrappedKeyRemanence,
    /// Reste in freigegebenen Speicherblöcken vor Wiederbelegung.
    FreedBlockRemanence,
    /// Reste überschriebener Keyslots.
    OldKeyslotRemanence,
    /// Flüchtigen Prozessspeicher.
    ProcessMemory,
    /// Rückschlüsse aus verbleibender Indextopologie.
    IndexTopologyInference,
    /// Textindex bis zum nächsten Segment-Rewrite.
    TextIndexUntilPurge,
    /// Replika-Synchronisationsfenster.
    ReplicaWindow,
    /// Externe Datensicherungen / Replikate.
    ExternalCopies,
    /// Externe Auftragsverarbeiter.
    ExternalProcessors,
    /// Kryptografisches Pseudonym-Commitment.
    PseudonymCommitment,
    /// Purge-Dauer skaliert mit Datenbankgröße.
    PurgeScalesWithDb,
    /// Verzögerter Purge-Lauf.
    PurgeDeferred,
    /// Schlüsselkompromittierung.
    KeyCompromise,
}

impl ResidualRisk {
    /// Feste Identifikator-Zeichenkette nach SPEC Anhang D.
    pub fn id(&self) -> &'static str {
        match self {
            Self::FlashRemanence => "RISK-FLASH-REMANENCE",
            Self::FsJournalOrCow => "RISK-FS-JOURNAL-OR-COW",
            Self::WrappedKeyRemanence => "RISK-WRAPPED-KEY-REMANENCE",
            Self::FreedBlockRemanence => "RISK-FREED-BLOCK-REMANENCE",
            Self::OldKeyslotRemanence => "RISK-OLD-KEYSLOT-REMANENCE",
            Self::ProcessMemory => "RISK-PROCESS-MEMORY",
            Self::IndexTopologyInference => "RISK-INDEX-TOPOLOGY-INFERENCE",
            Self::TextIndexUntilPurge => "RISK-TEXT-INDEX-UNTIL-PURGE",
            Self::ReplicaWindow => "RISK-REPLICA-WINDOW",
            Self::ExternalCopies => "RISK-EXTERNAL-COPIES",
            Self::ExternalProcessors => "RISK-EXTERNAL-PROCESSORS",
            Self::PseudonymCommitment => "RISK-PSEUDONYM-COMMITMENT",
            Self::PurgeScalesWithDb => "RISK-PURGE-SCALES-WITH-DB",
            Self::PurgeDeferred => "RISK-PURGE-DEFERRED",
            Self::KeyCompromise => "RISK-KEY-COMPROMISE",
        }
    }

    /// Statische Bedingung für Audit- und Verifikationswerkzeuge (AR-4).
    pub fn condition(&self) -> RiskCondition {
        match self {
            Self::FsJournalOrCow => RiskCondition::CowOrJournaling,
            Self::TextIndexUntilPurge | Self::PurgeDeferred => RiskCondition::UntilPurgeComplete,
            _ => RiskCondition::Always,
        }
    }
}

// =========================================================================
// Typisierter Zeuge (Typzustandsmuster, SPEC 8.1, 12.2, D-72, I-19, L-5, TY-10)
// =========================================================================

/// Typzustand: Scanner ist noch nicht scharfgeschaltet.
pub struct Unarmed;

/// Typzustand: Scanner ist scharfgeschaltet nach erfolgreicher Positivkontrolle.
pub struct Armed(());

/// Beleg für eine beobachtete Positivkontrolle (D-72).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ControlEvidence(());

impl ControlEvidence {
    /// Bezeugt das Auffinden des Kontrollmusters (nur in autorisierten Modulen aufgerufen).
    #[inline]
    pub const fn observed() -> Self {
        Self(())
    }
}

/// Rückstandsbefund bei fehlgeschlagener Verifikation (SPEC 12.2).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Residue {
    /// Anzahl gefundener Treffer / Rückstände.
    pub matches: u64,
}

impl fmt::Display for Residue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "erasure residue detected: {} matches found", self.matches)
    }
}

impl std::error::Error for Residue {}

/// Typisierter Scanner im Typzustandsmuster (TY-10, L-5).
#[must_use = "Scanner muss mit finish() abgeschlossen werden"]
pub struct Scanner<S> {
    layer: LayerId,
    class: MeasureClass,
    method_id: u16,
    files: u64,
    bytes: u64,
    _state: S,
}

impl Scanner<Unarmed> {
    /// Erzeugt einen neuen unbewaffneten Scanner.
    pub fn new(layer: LayerId, class: MeasureClass, method_id: u16) -> Self {
        Self {
            layer,
            class,
            method_id,
            files: 0,
            bytes: 0,
            _state: Unarmed,
        }
    }

    /// Einzige Brücke nach Armed; erfordert Positivkontrollbeleg (TY-10).
    pub fn arm(self, _ev: ControlEvidence) -> Scanner<Armed> {
        Scanner {
            layer: self.layer,
            class: self.class,
            method_id: self.method_id,
            files: self.files,
            bytes: self.bytes,
            _state: Armed(()),
        }
    }
}

impl Scanner<Armed> {
    /// Zeichnet geprüfte Dateien und Bytes auf.
    pub fn record(&mut self, files: u64, bytes: u64) {
        self.files = self.files.saturating_add(files);
        self.bytes = self.bytes.saturating_add(bytes);
    }

    /// Schließt die Messung ab.
    /// matches != 0 => Err(Residue).
    /// LayerWitness wird ausschließlich bei matches == 0 ausgegeben (TY-10, I-19, D-72).
    pub fn finish(
        self,
        matches: u64,
        duration: Duration,
        hash: [u8; 32],
    ) -> Result<LayerWitness, Residue> {
        if matches != 0 {
            return Err(Residue { matches });
        }

        Ok(LayerWitness {
            layer: self.layer,
            class: self.class,
            method_id: self.method_id,
            files: self.files,
            bytes: self.bytes,
            duration,
            hash,
        })
    }
}

/// Unveränderlicher Zeuge einer Schichtmessung (D-72, I-19).
/// Felder sind privat; matches() ist per Konstruktion immer 0.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct LayerWitness {
    layer: LayerId,
    class: MeasureClass,
    method_id: u16,
    files: u64,
    bytes: u64,
    duration: Duration,
    hash: [u8; 32],
}

impl LayerWitness {
    /// Liefert die Schicht-ID.
    #[inline]
    pub fn layer(&self) -> LayerId {
        self.layer
    }

    /// Liefert die Messklasse (MK1-MK3).
    #[inline]
    pub fn class(&self) -> MeasureClass {
        self.class
    }

    /// Liefert die Methoden-ID.
    #[inline]
    pub fn method_id(&self) -> u16 {
        self.method_id
    }

    /// Liefert die Anzahl geprüfter Dateien.
    #[inline]
    pub fn files(&self) -> u64 {
        self.files
    }

    /// Liefert die Anzahl geprüfter Bytes.
    #[inline]
    pub fn bytes(&self) -> u64 {
        self.bytes
    }

    /// Liefert die Scandauer.
    #[inline]
    pub fn duration(&self) -> Duration {
        self.duration
    }

    /// Liefert den kryptografischen Hash des Prüflaufs.
    #[inline]
    pub fn hash(&self) -> &[u8; 32] {
        &self.hash
    }

    /// matches() ist per Konstruktion immer 0 (I-19).
    #[inline]
    pub const fn matches(&self) -> u64 {
        0
    }
}
