//! Fehlerklassen und Hinweise (SP-PORTS §4, SPEC §3.5). Texte ohne Schlüssel- oder Nutzdaten.
use thiserror::Error as ThisError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ErrorClass {
    /// F-1: EINTR/EAGAIN, Backoff
    Transient,
    /// F-2: ENOSPC nur bei Vorallokation
    DiskFull,
    /// F-3: Sync-Fehler, Handle vergiftet
    Poisoned,
    /// Write/Sync wurde begonnen, Persistenz des Changes ist unbekannt
    Indeterminate,
    /// F-4: Prüfsumme, Siegel, Kette
    Corrupt,
    /// F-5: unbekannte Version oder Tag
    Unsupported,
    /// F-6: Schlüssel fehlt, offline oder verschreddert
    Key,
    /// Eingabe verletzt Grenze oder Vertrag
    Usage,
    /// Anker-Wettlauf verloren
    Conflict,
    /// Externer Anker liegt vor den wiederhergestellten dauerhaften Daten
    Rollback,
    /// interne Fehler, die nach Recovery/Neustart zu behandeln sind
    Internal,
}

#[derive(Debug, ThisError)]
#[non_exhaustive]
pub enum Error {
    #[error("transienter I/O-Fehler (Rolle {role}), Wiederholung mit Backoff")]
    Transient { role: &'static str },
    #[error("Speicher voll: Vorallokation fehlgeschlagen")]
    DiskFull,
    #[error("Handle vergiftet nach fehlgeschlagenem Sync ({cause})")]
    Poisoned { cause: &'static str },
    #[error("Commit-Ergebnis ist für Stamps {first_stamp}..={last_stamp} unbestimmt; Recovery/Idempotenz prüfen")]
    Indeterminate { first_stamp: u64, last_stamp: u64 },
    #[error("beschädigte Daten in {place}")]
    Corrupt { place: &'static str },
    #[error("nicht unterstützt: {what}")]
    Unsupported { what: &'static str },
    #[error("Schlüssel nicht verfügbar")]
    KeyUnavailable { retry_after_secs: Option<u64> },
    #[error("Schlüsselverlust ohne bestätigten Löschbeleg")]
    KeyLost,
    #[error("Daten sind gelöscht (Crypto-Shredding)")]
    Shredded,
    #[error("Cluster ist retired")]
    ClusterRetired,
    #[error("Cluster ist unbekannt")]
    UnknownCluster,
    #[error("Sicht liegt unterhalb des verfügbaren Historienbodens")]
    BelowFloor,
    #[error("View-Lease abgelaufen bei Stamp {stamp} (Bodenhinweis {floor_hint})")]
    SnapshotTooOld { stamp: u64, floor_hint: u64 },
    #[error("View wurde widerrufen")]
    Revoked,
    #[error("Daten wurden ab Stamp {at} gelöscht")]
    Erased { at: u64 },
    #[error("Löschabsicht auf vergiftetem Handle vorgemerkt; Neustart schließt Gate")]
    ErasePending,
    #[error("Zugriff ist durch eine Restriction gesperrt")]
    Restricted,
    #[error("Guard-Konflikt")]
    Conflict,
    #[error("Guard-Snapshot ist veraltet; mit frischer Sicht erneut validieren")]
    GuardRetry,
    #[error("Commit-Pipeline ausgelastet; später erneut versuchen")]
    Backpressure,
    #[error("Change überschreitet das zulässige Formatlimit")]
    ChangeTooLarge,
    #[error("Idempotenz-Token wurde mit anderem Inhalt wiederverwendet")]
    IdemConflict,
    #[error("Idempotenzspeicher ist ausgelastet")]
    IdemCapacity,
    #[error("Branch ist geschlossen")]
    BranchClosed,
    #[error("Branch hat noch lebende Kinder")]
    BranchHasChildren,
    #[error("ungültiger Fork-Punkt für Branch")]
    InvalidForkPoint,
    #[error("Consumer muss nach einem Lag neu aufbauen")]
    Lagged,
    #[error("Abgeleiteter Index liegt hinter dem View-Stamp")]
    IndexLag,
    #[error("Erase-Hülle überschreitet das konfigurierte Fanout-Limit")]
    EraseFanoutExceeded,
    #[error("Uhrabweichung verhindert sichere Fristberechnung")]
    ClockUncertain,
    #[error("Space-ID ist dem Systemnamensraum vorbehalten")]
    ReservedSpace,
    #[error("monotone ID- oder Stamp-Reserve ist ausgeschöpft")]
    Exhausted,
    #[error("Eingabe verletzt Grenze: {limit}")]
    Limit { limit: &'static str },
    #[error("Ressourcen-Admission verweigert ({resource})")]
    AdmissionLimit { resource: &'static str },
    #[error("Dateideskriptor-Limit zu niedrig (benötigt {needed})")]
    FdLimitTooLow { needed: u64 },
    #[error("Anker-Wettlauf verloren (CAS)")]
    CasLost,
    #[error("Rollback erkannt: Anker liegt vor Catalog oder Ledger")]
    Rollback,
    #[error("Erforderlicher externer Anker nicht verfügbar")]
    AnchorUnavailable,
    #[error("Erase-Ledger nicht verfügbar")]
    LedgerUnavailable,
    #[error("Erase-Ledger darf im Produktionsprofil nicht auf demselben Gerät wie Daten liegen")]
    LedgerSameDevice,
    #[error("Backup liegt hinter dem konfigurierten Horizont (Floor {floor}, Mark {mark})")]
    BackupBeyondHorizon { floor: u64, mark: u64 },
    #[error("Schlüsselprovider-Zusicherung für Produktionsprofil unzureichend")]
    ProviderAssuranceInsufficient,
    #[error("I/O-Fehler (Rolle {role}, Art {kind:?})")]
    Io {
        role: &'static str,
        kind: std::io::ErrorKind,
    },
}

impl Error {
    pub fn class(&self) -> ErrorClass {
        match self {
            Self::Transient { .. } | Self::Backpressure | Self::IndexLag => ErrorClass::Transient,
            Self::Io { kind: std::io::ErrorKind::Interrupted, .. } => ErrorClass::Transient,
            Self::Io { .. } => ErrorClass::Internal,
            Self::DiskFull => ErrorClass::DiskFull,
            Self::Poisoned { .. } => ErrorClass::Poisoned,
            Self::Indeterminate { .. } => ErrorClass::Indeterminate,
            Self::Corrupt { .. } => ErrorClass::Corrupt,
            Self::Unsupported { .. } => ErrorClass::Unsupported,
            Self::KeyUnavailable { .. }
            | Self::KeyLost
            | Self::Shredded
            | Self::ProviderAssuranceInsufficient => ErrorClass::Key,
            Self::Limit { .. }
            | Self::AdmissionLimit { .. }
            | Self::FdLimitTooLow { .. }
            | Self::ClusterRetired
            | Self::UnknownCluster
            | Self::BelowFloor
            | Self::SnapshotTooOld { .. }
            | Self::Revoked
            | Self::Restricted
            | Self::ChangeTooLarge
            | Self::IdemCapacity
            | Self::BranchClosed
            | Self::BranchHasChildren
            | Self::InvalidForkPoint
            | Self::Lagged
            | Self::EraseFanoutExceeded
            | Self::ClockUncertain
            | Self::Erased { .. }
            | Self::ErasePending
            | Self::ReservedSpace
            | Self::Exhausted
            | Self::LedgerUnavailable
            | Self::LedgerSameDevice
            | Self::AnchorUnavailable => ErrorClass::Usage,
            Self::CasLost | Self::Conflict | Self::IdemConflict | Self::GuardRetry => {
                ErrorClass::Conflict
            }
            Self::Rollback | Self::BackupBeyondHorizon { .. } => ErrorClass::Rollback,
        }
    }

    pub fn hint(&self) -> &'static str {
        match self {
            Self::Transient { .. } | Self::Io { .. } => "Mit Backoff erneut versuchen",
            Self::DiskFull => "Platz schaffen oder Erase-Reserve prüfen",
            Self::GuardRetry => "Mit einer frischen Sicht erneut validieren",
            Self::Backpressure => "Mit kurzem Backoff erneut versuchen",
            Self::Poisoned { .. } => "Speicher neu öffnen; Wiederholung nie auf demselben Handle",
            Self::Indeterminate { .. } => {
                "Store neu öffnen und denselben IdemToken zum Auflösen wiederverwenden"
            }
            Self::Corrupt { .. } => {
                "Integritätsbericht prüfen, Datei aus Sicherung wiederherstellen"
            }
            Self::Unsupported { .. } => "Version des Kerns prüfen",
            Self::KeyUnavailable { .. } => "Schlüsselprovider prüfen",
            Self::KeyLost => "Store quarantänisieren und Erase-Ledger prüfen",
            Self::Shredded => "Daten sind endgültig gelöscht",
            Self::ProviderAssuranceInsufficient => {
                "Hardware/KMS-Provider nutzen oder acknowledged_weak_keys explizit setzen"
            }
            Self::ClusterRetired => "Erasure-Ticket prüfen",
            Self::UnknownCluster => "Cluster-Registrierung prüfen",
            Self::BelowFloor => "Neuere View öffnen oder Historie wiederherstellen",
            Self::SnapshotTooOld { .. } => {
                "Neue View öffnen und mit ResumeToken per view_at fortsetzen"
            }
            Self::Revoked => "Neue View öffnen",
            Self::Erased { .. } => {
                "Löschstatus beachten; die Sicht enthält keine Daten dieses Clusters"
            }
            Self::ErasePending => "Prozess neu starten, um die vorgemerkte Löschung zu gaten",
            Self::Restricted => "Restriction und zuständige Freigabe prüfen",
            Self::Conflict | Self::CasLost | Self::IdemConflict => {
                "Zustand neu lesen und Request prüfen"
            }
            Self::ChangeTooLarge => "Change in kleinere atomare Änderungen aufteilen",
            Self::IdemCapacity => {
                "Ablauf/Trim abwarten; Request nicht ohne Token wiederholen"
            }
            Self::BranchClosed => "Einen neuen Branch vom aktuellen Head anlegen",
            Self::BranchHasChildren => "Zuerst die Branch-Kinder schließen",
            Self::InvalidForkPoint => "ForkPoint innerhalb von history_floor..=parent_head wählen",
            Self::Lagged => "Consumer ab einem vollständigen Snapshot neu aufbauen",
            Self::IndexLag => "Delta-Abfrage nutzen oder auf Index-Watermark warten",
            Self::EraseFanoutExceeded => "Löschdomäne verkleinern oder Grenze prüfen",
            Self::ClockUncertain => "Uhrquelle prüfen; Fristen bleiben bis dahin angehalten",
            Self::ReservedSpace => "Space-IDs 0 bis 255 sind dem System vorbehalten",
            Self::Exhausted => "ID-Raum ist erschöpft; IDs werden niemals wiederverwendet",
            Self::Limit { .. } | Self::AdmissionLimit { .. } => {
                "Eingabe oder Ressourcenbudget prüfen"
            }
            Self::FdLimitTooLow { .. } => {
                "RLIMIT_NOFILE erhöhen oder max_open_files reduzieren"
            }
            Self::Rollback => "ReadOnly-Diagnose öffnen und Restore-Reconcile ausführen",
            Self::AnchorUnavailable => "Externen Ankerdienst prüfen",
            Self::LedgerUnavailable => "Ledger-Verzeichnis und Schreib-/Lesbarkeit prüfen",
            Self::LedgerSameDevice => {
                "ledger_dir auf eigenes Gerät legen oder ledger_same_device_ack setzen"
            }
            Self::BackupBeyondHorizon { .. } => {
                "Kopie liegt hinter dem Backup-Horizont; neuere Sicherung verwenden"
            }
        }
    }
}
