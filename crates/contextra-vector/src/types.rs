// FILE-CONTEXT
// ZWECK: Lokale Typ- und Traitabstraktionen für Vektorsuche und Indexierung (Klasse A).
// INVARIANTEN: Zero-Panic, DocId-Breite konfigurierbar (64/128-Bit), deterministischer RNG.
// HOTSPOTS: DocId::from_key, SeededRng, SystemClock
// STAND: TS:2026-11-11T02:00:00Z

//! `types` — Lokale Typ- und Traitabstraktionen für Vektorsuche und Indexierung (Klasse A).
//!
//! # Zweck
//! Stellt sicher, dass `contextra-vector` als autarker Baustein ohne interne Abhängigkeiten
//! zu `contextra-ports`, `contextra-types` oder `contextra-mvcc` funktioniert (SPEC v4.0 §4, B1–B4).
//! Bei aktiviertem transitionalen Feature `ports-compat` stehen verlustfreie Konvertierungen
//! zu den Workspace-Typen bereit.

use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

// SECTION Identifikatoren (DocId, TxId)
pub use contextra_ids::{DocId, TxId, TOMBSTONE_BIT};
// !SECTION

// SECTION Metriken & Suchergebnisse

// ANCHOR[contextra-vector:TYPES-METRIC]
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

// ANCHOR[contextra-vector:TYPES-SCORED-DOC]
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

// !SECTION

// SECTION Umwelt-Abstraktionen (Clock, Rng, Metrics, Maintain)

// ANCHOR[contextra-vector:TYPES-CLOCK]
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

// ANCHOR[contextra-vector:TYPES-RNG]
/// Trait für deterministische Zufallszahlen im Vektorindex.
pub trait Rng: Send + Sync {
    /// Liefert ein zufälliges `u64`.
    fn next_u64(&self) -> u64;
    /// Liefert ein gleichverteiltes `f64` im Intervall `[0.0, 1.0)`.
    fn next_f64(&self) -> f64;
    /// Liefert eine normalisierte Zufallszahl `[0.0, 1.0)`.
    fn next_unit_f64(&self) -> f64 {
        self.next_f64()
    }
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

// ANCHOR[contextra-vector:TYPES-METRICS-SINK]
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

// ANCHOR[contextra-vector:TYPES-MAINTAIN]
/// Neutraler Wartungs-Trait für Speicherfreigaben und Kompaktierung (SPEC §6.1).
pub trait Maintain: Send + Sync {
    /// Führt physische Bereinigung und Slot-Freigaben durch.
    fn compact(&self) -> crate::error::Result<()>;
    /// Prüft den Rebuild- bzw. Reorganisationsbedarf.
    fn is_rebuild_required(&self) -> bool;
}
// !SECTION
