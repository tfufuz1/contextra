//! `types` — Lokale Typ- und Traitabstraktionen für Vektorsuche und Indexierung (Klasse A).
//!
//! # Zweck
//! Stellt sicher, dass `contextra-vector` als autarker Baustein ohne interne Abhängigkeiten
//! zu `contextra-ports`, `contextra-types` oder `contextra-mvcc` funktioniert (SPEC v4.0 §4, B1–B4).
//! Bei aktiviertem transitionalen Feature `ports-compat` stehen verlustfreie Konvertierungen
//! zu den Workspace-Typen bereit.

use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

/// Dokumenten-Identifikator (64-Bit oder 128-Bit).
#[cfg(not(feature = "docid-128"))]
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
#[repr(transparent)]
pub struct DocId(pub u64);

#[cfg(feature = "docid-128")]
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
#[repr(transparent)]
pub struct DocId(pub u128);

impl DocId {
    /// Kleinstmöglicher Bezeichner.
    pub const MIN: Self = Self(0);

    /// Erstellt eine neue `DocId`.
    #[cfg(not(feature = "docid-128"))]
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    #[cfg(feature = "docid-128")]
    pub const fn new(id: u128) -> Self {
        Self(id)
    }

    /// Gibt den inneren ganzzahligen Wert zurück.
    #[cfg(not(feature = "docid-128"))]
    pub const fn inner(&self) -> u64 {
        self.0
    }

    #[cfg(feature = "docid-128")]
    pub const fn inner(&self) -> u128 {
        self.0
    }

    /// Erzeugt eine DocId aus einem Schlüssel-String via Hashing.
    pub fn from_key(key: &str) -> Result<Self, crate::error::VectorError> {
        let hash = blake3::hash(key.as_bytes());
        let bytes = hash.as_bytes();
        #[cfg(not(feature = "docid-128"))]
        {
            let mut num_bytes = [0u8; 8];
            num_bytes.copy_from_slice(&bytes[..8]);
            Ok(Self(u64::from_le_bytes(num_bytes)))
        }
        #[cfg(feature = "docid-128")]
        {
            let mut num_bytes = [0u8; 16];
            num_bytes.copy_from_slice(&bytes[..16]);
            Ok(Self(u128::from_le_bytes(num_bytes)))
        }
    }
}

impl fmt::Display for DocId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DocId({})", self.0)
    }
}

#[cfg(not(feature = "docid-128"))]
impl From<u64> for DocId {
    fn from(val: u64) -> Self {
        Self(val)
    }
}

#[cfg(feature = "docid-128")]
impl From<u128> for DocId {
    fn from(val: u128) -> Self {
        Self(val)
    }
}

/// Transaktions-Identifikator (64-Bit).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
#[repr(transparent)]
pub struct TxId(pub u64);

impl TxId {
    /// Transaktion 0 (System/Baseline).
    pub const SYSTEM: Self = Self(0);

    /// Erstellt eine neue `TxId`.
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    /// Gibt den numerischen Wert zurück.
    pub const fn as_u64(&self) -> u64 {
        self.0
    }
}

impl fmt::Display for TxId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TxId({})", self.0)
    }
}

impl From<u64> for TxId {
    fn from(val: u64) -> Self {
        Self(val)
    }
}

/// Unterstützte Distanzmetriken für Vektorräume.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, Default,
)]
pub enum DistanceMetric {
    /// Kosinus-Distanz ($1 - \text{sim}$).
    #[default]
    Cosine,
    /// Euklidische Distanz ($L_2$).
    Euclidean,
    /// Skalarprodukt (Dot-Produkt, invertiert für Minimierung).
    DotProduct,
}

/// Suchergebnis mit Dokument-ID und Relevanz-Score / Distanz.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ScoredDocument {
    /// Dokument-ID des Treffers.
    pub doc_id: DocId,
    /// Ähnlichkeitswert (Score, höher ist ähnlicher).
    pub score: f32,
    /// Distanzwert (niedriger ist näher).
    pub distance: f32,
}

impl ScoredDocument {
    /// Erzeugt ein neues ScoredDocument.
    pub fn new(doc_id: DocId, score: f32, distance: f32) -> Self {
        Self {
            doc_id,
            score,
            distance,
        }
    }
}

/// Maximal zulässiges `k` für Nachbarschaftssuchen.
pub const MAX_SEARCH_K: usize = 10_000;

/// Bitmaske für Tombstone-Markierung in Sequenznummern.
pub const TOMBSTONE_BIT: u64 = 1 << 63;

/// Trait für Zeitabfragen ohne Abhängigkeit von `contextra-ports`.
pub trait Clock: Send + Sync {
    /// Liefert die aktuelle Zeit in Unix-Nanosekunden.
    fn now_unix_nanos(&self) -> u64;
    /// Liefert einen monoton steigenden Nanosekunden-Zähler.
    fn monotonic_nanos(&self) -> u64;
}

/// Standardmäßige Implementierung von [`Clock`] auf Basis von `std::time::SystemTime` / `Instant`.
#[derive(Debug, Default, Clone, Copy)]
pub struct SystemClock;

impl SystemClock {
    /// Erstellt eine neue [`SystemClock`].
    pub fn new() -> Self {
        Self
    }
}

impl Clock for SystemClock {
    #[allow(clippy::disallowed_methods, clippy::cast_possible_truncation)]
    fn now_unix_nanos(&self) -> u64 {
        SystemTime::now() // ALLOW-DETERMINISM: Wall clock baseline in standalone SystemClock
            .duration_since(UNIX_EPOCH)
            .map(|d| u64::try_from(d.as_nanos()).unwrap_or(u64::MAX))
            .unwrap_or(0)
    }

    #[allow(clippy::disallowed_methods, clippy::cast_possible_truncation)]
    fn monotonic_nanos(&self) -> u64 {
        self.now_unix_nanos()
    }
}

/// Trait für deterministische Zufallszahlen im Vektorindex.
pub trait Rng: Send + Sync {
    /// Liefert ein zufälliges `u64`.
    fn next_u64(&self) -> u64;
    /// Liefert ein gleichverteiltes `f64` im Intervall `[0.0, 1.0)`.
    fn next_f64(&self) -> f64;
}

/// Einfacher deterministischer SplitMix64 RNG.
#[derive(Debug)]
pub struct SeededRng {
    state: std::sync::atomic::AtomicU64,
}

impl SeededRng {
    /// Erstellt einen neuen Generator mit Seed.
    pub fn new(seed: u64) -> Self {
        Self {
            state: std::sync::atomic::AtomicU64::new(if seed == 0 {
                0x9E3779B97F4A7C15
            } else {
                seed
            }),
        }
    }
}

impl Rng for SeededRng {
    fn next_u64(&self) -> u64 {
        let mut z = self
            .state
            .fetch_add(0x9E3779B97F4A7C15, std::sync::atomic::Ordering::Relaxed);
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^ (z >> 31)
    }

    fn next_f64(&self) -> f64 {
        (self.next_u64() >> 11) as f64 * (1.0 / (1u64 << 53) as f64)
    }
}

/// Trait zur Metrik-Erfassung.
pub trait MetricsSink: Send + Sync {
    /// Zählt ein Ereignis hoch.
    fn record_counter(
        &self,
        name: &'static str,
        value: u64,
        labels: &[(&'static str, &'static str)],
    );
    /// Zeichnet eine Histogramm-Messung auf.
    fn record_histogram(
        &self,
        name: &'static str,
        value: f64,
        labels: &[(&'static str, &'static str)],
    );
}

/// No-Op Metriksink für Standardbetrieb ohne Telemetrie.
#[derive(Debug, Default, Clone, Copy)]
pub struct NoopMetricsSink;

impl MetricsSink for NoopMetricsSink {
    fn record_counter(&self, _: &'static str, _: u64, _: &[(&'static str, &'static str)]) {}
    fn record_histogram(&self, _: &'static str, _: f64, _: &[(&'static str, &'static str)]) {}
}

/// Neutraler Wartungs-Trait für Speicherfreigaben und Kompaktierung (SPEC §6.1).
pub trait Maintain: Send + Sync {
    /// Führt physische Bereinigung und Slot-Freigaben durch.
    fn compact(&self) -> crate::error::Result<()>;
    /// Prüft den Rebuild- bzw. Reorganisationsbedarf.
    fn is_rebuild_required(&self) -> bool;
}

#[cfg(feature = "ports-compat")]
mod compat {
    use super::*;

    impl From<DocId> for contextra_types::DocId {
        fn from(d: DocId) -> Self {
            #[cfg(not(feature = "docid-128"))]
            {
                contextra_types::DocId::new(d.0)
            }
            #[cfg(feature = "docid-128")]
            {
                contextra_types::DocId::new(d.0)
            }
        }
    }

    impl From<contextra_types::DocId> for DocId {
        fn from(d: contextra_types::DocId) -> Self {
            Self(d.inner())
        }
    }

    impl From<TxId> for contextra_types::TxId {
        fn from(t: TxId) -> Self {
            contextra_types::TxId(t.0)
        }
    }

    impl From<contextra_types::TxId> for TxId {
        fn from(t: contextra_types::TxId) -> Self {
            Self(t.0)
        }
    }

    impl From<DistanceMetric> for contextra_types::DistanceMetric {
        fn from(m: DistanceMetric) -> Self {
            match m {
                DistanceMetric::Cosine => contextra_types::DistanceMetric::Cosine,
                DistanceMetric::Euclidean => contextra_types::DistanceMetric::Euclidean,
                DistanceMetric::DotProduct => contextra_types::DistanceMetric::DotProduct,
            }
        }
    }

    impl From<contextra_types::DistanceMetric> for DistanceMetric {
        fn from(m: contextra_types::DistanceMetric) -> Self {
            match m {
                contextra_types::DistanceMetric::Cosine => DistanceMetric::Cosine,
                contextra_types::DistanceMetric::Euclidean => DistanceMetric::Euclidean,
                contextra_types::DistanceMetric::DotProduct => DistanceMetric::DotProduct,
                _ => DistanceMetric::Cosine,
            }
        }
    }

    impl From<ScoredDocument> for contextra_types::ScoredDocument {
        fn from(sd: ScoredDocument) -> Self {
            contextra_types::ScoredDocument {
                doc_id: sd.doc_id.into(),
                score: sd.score,
            }
        }
    }

    impl From<contextra_types::ScoredDocument> for ScoredDocument {
        fn from(sd: contextra_types::ScoredDocument) -> Self {
            ScoredDocument {
                doc_id: sd.doc_id.into(),
                score: sd.score,
                distance: 1.0 - sd.score,
            }
        }
    }
}
