//! # Katalog-Datentypen und Metadaten
//!
//! Definitionen für Datei-Metadaten, Space-Optionen und Halteregeln nach SPEC §9.7.

use std::fmt;

/// Kennung für die Art einer Katalog-Datei.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum CatalogFileKind {
    /// SSTable-Run (kind 1).
    Run = 1,
    /// Blob-Datei (kind 2).
    Blob = 2,
    /// Segment-Datei (kind 3).
    Segment = 3,
}

impl CatalogFileKind {
    /// Erzeugt die Kind-Kennung aus dem numerischen u8-Wert.
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            1 => Some(Self::Run),
            2 => Some(Self::Blob),
            3 => Some(Self::Segment),
            _ => None,
        }
    }

    /// Liefert den numerischen u8-Wert.
    pub fn to_u8(self) -> u8 {
        match self {
            Self::Run => 1,
            Self::Blob => 2,
            Self::Segment => 3,
        }
    }
}

/// Metadaten einer Datei im Katalog nach SPEC §9.7 (Feld-ID 0x0001).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileMeta {
    /// Dateiart (Run, Blob, Segment).
    pub kind: CatalogFileKind,
    /// Eindeutige Datei-ID (für Blobs: blob_no).
    pub id: u64,
    /// Zugehörige Space-ID.
    pub space: u16,
    /// LSM-Level (0 für L0, >0 für tiefere Level).
    pub level: u8,
    /// Speicher-Tier (0 = Local, 1 = Cold).
    pub tier: u8,
    /// Datei-Flags.
    pub flags: u16,
    /// Minimaler Stamp in dieser Datei.
    pub min_stamp: u64,
    /// Maximaler Stamp in dieser Datei.
    pub max_stamp: u64,
    /// Dateigröße in Bytes.
    pub bytes: u64,
    /// Anzahl der Einträge in dieser Datei.
    pub entries: u64,
    /// Generationsnummer (wichtig für Blob-Sweep).
    pub generation: u32,
    /// Benutzerdefinierte Segment-Art (falls Segment).
    pub segment_kind: u16,
    /// Kleinste referenzierte Blob-Nummer (0 falls keine).
    pub blob_no_min: u64,
    /// Größte referenzierte Blob-Nummer (0 falls keine).
    pub blob_no_max: u64,
    /// Kleinster gehaltener Schlüssel.
    pub min_key: Vec<u8>,
    /// Größter gehaltener Schlüssel.
    pub max_key: Vec<u8>,
}

/// Konfigurationsoptionen eines Space (Feld-ID 0x0008, SPEC §6.4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogSpaceOptions {
    /// Feste Headerlänge je Eintrag (0..255).
    pub header_len: u8,
    /// Anzahl der Gewichte (bis zu 4).
    pub weights_k: u8,
    /// Space-Flags (separate_body, domain_sealed, etc.).
    pub flags: u16,
    /// Schwelle für getrennte Auslagerung von Wertkörpern in Blobs.
    pub separate_threshold: u32,
    /// Präfixfilter-Länge.
    pub prefix_filter: u8,
    /// Lokalitätspräfix-Länge.
    pub locality_prefix: u8,
}

/// Repräsentation eines Schlüsselbereichs für Erase-Operationen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogKeyRange {
    /// Startschlüssel (inklusiv).
    pub start: Vec<u8>,
    /// Endschlüssel (exklusiv; None = unbeschränkt bis Space-Ende).
    pub end: Option<Vec<u8>>,
}

/// Metadaten eines dauerhaften Holds (Feld-ID 0x0012).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HoldMeta {
    /// Eindeutige Hold-ID.
    pub hold_id: u64,
    /// Gehaltener Mindeststempel.
    pub stamp: u64,
    /// Eigentümer-Kennung.
    pub owner: Vec<u8>,
    /// Ablaufzeitpunkt in Unix-Millisekunden (0 = unbegrenzt).
    pub expires_ms: u64,
}

/// Metadaten einer anhängigen physischen Löschung (Feld-ID 0x000D).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErasePendingMeta {
    /// Lösch-Auftrags-ID (= Commit-Stempel S).
    pub id: u64,
    /// Betroffener Space.
    pub space: u16,
    /// Betroffener Schlüsselbereich.
    pub range: CatalogKeyRange,
    /// Maximales betroffenes LSM-Level.
    pub level: u8,
}

/// Metadaten eines bereits gelöschten Bereichs (Feld-ID 0x0010/0x0011).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErasedRangeMeta {
    /// Space-ID.
    pub space: u16,
    /// Gelöschter Schlüsselbereich.
    pub range: CatalogKeyRange,
    /// Lösch-Stempel S.
    pub stamp_s: u64,
}

/// Fehlerarten im Katalog.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CatalogError {
    /// Beschädigter Katalog-Record oder Datei (INV-CAT-3).
    Corrupt(&'static str),
    /// Unbekanntes Pflichtfeld angetroffen.
    UnsupportedField(u16),
    /// Ungültiges Datenformat im Feld.
    InvalidFormat(&'static str),
    /// Unerwartetes Dateiende.
    UnexpectedEof,
    /// Allgemeiner E/A-Fehler.
    Io(String),
}

impl fmt::Display for CatalogError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Corrupt(msg) => write!(f, "INV-CAT-3: Katalogdaten beschädigt: {msg}"),
            Self::UnsupportedField(id) => write!(f, "Nicht unterstütztes Pflichtfeld: {id:#06x}"),
            Self::InvalidFormat(msg) => write!(f, "Ungültiges Format im Katalog-Record: {msg}"),
            Self::UnexpectedEof => write!(f, "Unerwartetes Dateiende im Katalog"),
            Self::Io(msg) => write!(f, "Katalog E/A-Fehler: {msg}"),
        }
    }
}

impl std::error::Error for CatalogError {}

impl From<std::io::Error> for CatalogError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e.to_string())
    }
}
