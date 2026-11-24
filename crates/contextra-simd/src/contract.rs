// FILE-CONTEXT
// ZWECK: Vertragsdatei für contextra-simd (Eingefrorener Crate-Vertrag Klasse A).
// INVARIANTEN: Zero-Panic, autarker Baustein ohne interne Pflicht-Dependencies (SPEC §4.3).
// HOTSPOTS: DistanceMetric, DistanceFn, SimdError, Result
// STAND: 2026-11-11

//! Vertragsdatei für `contextra-simd` (Eingefrorener Crate-Vertrag Klasse A).
//!
//! Definiert die kanonischen Typen und Schnittstellen für die autarke Nutzung
//! des SIMD-Distanzkern-Bausteins gemäß SPEC §4.3 (B1–B4).

pub use crate::error::{Error, Result, SimdError};
pub use crate::metric::DistanceMetric;

// SECTION CONTRACT-DISTANCE-FN
// ANCHOR[contextra-simd:CONTRACT-DISTANCE-FN]
/// Abstrakte Distanzfunktions-Schnittstelle zur dynamischen oder externen Injektion von Berechnungen.
///
/// Ermöglicht es Vektor-Konsumenten (wie `contextra-vector`), beliebige Distanz-Backends
/// zustandslos und `Send + Sync` einzubinden.
pub trait DistanceFn: Send + Sync {
    /// Berechnet die Distanz zwischen zwei gleich langen `f32`-Vektorslices.
    ///
    /// # Fehler
    /// Liefert `Err(SimdError)`, falls Dimensionen nicht übereinstimmen oder NaN/Inf vorliegt.
    fn distance(&self, a: &[f32], b: &[f32]) -> Result<f32>;
}

/// Implementierung von [`DistanceFn`] für das Dispatcher-Gesamtwerkzeug mit konfigurierter Metrik.
#[derive(Debug, Clone, Copy)]
pub struct MetricDistanceFn {
    /// Die auszuführende Distanzmetrik.
    pub metric: DistanceMetric,
    /// Der zugrundeliegende Dispatcher.
    pub dispatcher: crate::dispatch::Dispatcher,
}

impl MetricDistanceFn {
    /// Erstellt eine neue Distanzfunktion mit der angegebenen Metrik und Auto-Dispatch.
    #[inline]
    pub fn new(metric: DistanceMetric) -> Self {
        Self {
            metric,
            dispatcher: crate::dispatch::Dispatcher::auto(),
        }
    }

    /// Erstellt eine neue Distanzfunktion mit benutzerdefiniertem Dispatcher.
    #[inline]
    pub const fn with_dispatcher(metric: DistanceMetric, dispatcher: crate::dispatch::Dispatcher) -> Self {
        Self { metric, dispatcher }
    }
}

impl DistanceFn for MetricDistanceFn {
    #[inline]
    fn distance(&self, a: &[f32], b: &[f32]) -> Result<f32> {
        match self.metric {
            DistanceMetric::Cosine => self.dispatcher.cosine_distance(a, b),
            DistanceMetric::Euclidean => self.dispatcher.euclidean_distance(a, b),
            DistanceMetric::DotProduct => self.dispatcher.dot_product_distance(a, b),
        }
    }
}
// !SECTION CONTRACT-DISTANCE-FN
