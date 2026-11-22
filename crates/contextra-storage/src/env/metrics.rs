//! # Metrics Adapter
//!
//! SPEC §4, §18 (INV-API-1):
//! Adapter zum `Metrics`-Port aus `contextra-storage-ports`.
//! Metrikmethoden nehmen ausschließlich `&'static str` und Zahlen (`u64`, `i64`, `f64`).
//! Schlüssel- oder Wertbytes können strukturell nicht übergeben werden (INV-API-1).

use contextra_storage_ports::Metrics;
use std::sync::Arc;

/// No-Op-Implementierung für den Fall, dass kein Metrikensammler übergeben wurde.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoopMetrics;

impl Metrics for NoopMetrics {
    fn counter(&self, _n: &'static str, _v: u64) {}
    fn gauge(&self, _n: &'static str, _v: i64) {}
    fn observe(&self, _n: &'static str, _v: f64) {}
}

/// Strenger Adapter zum `Metrics`-Port.
///
/// Garantiert INV-API-1: Metrikennamen sind statische Zeichenketten,
/// Werte sind ausschließlich primitive Zahlen.
#[derive(Clone)]
pub struct MetricsAdapter {
    sink: Arc<dyn Metrics>,
}

impl MetricsAdapter {
    /// Erstellt einen neuen Adapter mit der übergebenen Metriken-Senke.
    pub fn new(sink: Arc<dyn Metrics>) -> Self {
        Self { sink }
    }

    /// Erstellt einen No-Op-Adapter.
    pub fn noop() -> Self {
        Self {
            sink: Arc::new(NoopMetrics),
        }
    }

    /// Erhöht einen Zähler um `delta`.
    ///
    /// INV-API-1: Nur &'static str und Zahlen, keine Byte-Inhalte.
    pub fn count(&self, name: &'static str, delta: u64) {
        self.sink.counter(name, delta);
    }

    /// Inkrementiert einen Zähler um 1.
    ///
    /// INV-API-1: Nur &'static str und Zahlen.
    pub fn inc(&self, name: &'static str) {
        self.sink.counter(name, 1);
    }

    /// Setzt einen Pegelwert.
    ///
    /// INV-API-1: Nur &'static str und Zahlen, keine Byte-Inhalte.
    pub fn gauge(&self, name: &'static str, val: i64) {
        self.sink.gauge(name, val);
    }

    /// Protokolliert eine Verteilungsmessung.
    ///
    /// INV-API-1: Nur &'static str und Zahlen, keine Byte-Inhalte.
    pub fn observe(&self, name: &'static str, val: f64) {
        self.sink.observe(name, val);
    }
}
