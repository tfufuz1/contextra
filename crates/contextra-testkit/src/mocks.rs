//! # Mock-Implementierungen für Tests (`mocks`)
//!
//! Dieses Modul stellt Test-Doppelgänger (Mocks) für Ports bereit, die in Testumgebungen
//! deterministisches Verhalten ohne externe Dienste sicherstellen:
//! - [`MockEmbedder`]: Liefert deterministische Vektor-Embeddings für Textanalyse.
//! - [`TestMetricsSink`]: Zeichnet Metrik-Ereignisse (Counter, Gauge, Histogram) im Speicher auf.
//! - [`MetricEvent`]: Repräsentiert ein einzelnes aufgezeichnetes Metrik-Ereignis.
//!
//! ## Ring & Architektur-Einordnung
//! - **Ring:** Werkzeug / Tooling
//! - **Implementierte Ports:**
//!   - [`contextra_ports::EmbeddingProvider`]
//!   - [`contextra_ports::MetricsSink`]
//!
//! ## Invarianten
//! - Kein `unsafe` (`#![forbid(unsafe_code)]`).
//! - Thread-sichere Aufzeichnung von Metriken über Mutex-geschützte Ereignislisten.

// FILE-CONTEXT
// STAND: 2026-10-25
// ZWECK: Mock implementations for testing (MockEmbedder, TestMetricsSink, MetricEvent).
// INVARIANTEN: No unsafe code allowed in testkit (#![forbid(unsafe_code)]).

use contextra_ports::{BoxFuture, EmbeddingError, EmbeddingProvider, MetricsSink};

// ANCHOR[contextra-testkit:MOCK-EMBEDDER-STRUCT-001]
/// Deterministischer Mock-Embedding-Provider für Testläufe.
///
/// **Zweck:** Ermöglicht semantische Vektortests ohne teure oder nicht-deterministische
/// externe KI-/ML-Modelle oder Inferenz-Engines.
///
/// **Verhalten:**
/// Gibt entweder einen statischen Vektor (`fixed_output`) oder einen Default-Vektor
/// aus wiederholten `0.1f32`-Werten der gewünschten Dimension zurück.
pub struct MockEmbedder {
    /// Ziel-Dimensionalität der generierten Embeddings.
    pub dim: usize,
    /// Fester Ausgabevektor für reproduzierbare Assertions in Tests.
    pub fixed_output: Option<Vec<f32>>,
}

impl MockEmbedder {
    // ANCHOR[contextra-testkit:MOCK-EMBEDDER-NEW-001]
    /// Erzeugt einen `MockEmbedder` mit der vorgegebenen Dimension.
    ///
    /// Standardmäßig wird ein Vektor aus `0.1f32` der Länge `dim` erzeugt.
    pub fn new(dim: usize) -> Self {
        Self {
            dim,
            fixed_output: None,
        }
    }

    // ANCHOR[contextra-testkit:MOCK-EMBEDDER-FIXED-001]
    /// Erzeugt einen `MockEmbedder` mit einem festen Vektor.
    ///
    /// Die Dimension wird automatisch aus der Länge von `fixed_output` abgeleitet.
    pub fn with_fixed_output(fixed_output: Vec<f32>) -> Self {
        let dim = fixed_output.len();
        Self {
            dim,
            fixed_output: Some(fixed_output),
        }
    }
}

// ANCHOR[contextra-testkit:MOCK-EMBEDDER-TRAIT-IMPL-001]
impl EmbeddingProvider for MockEmbedder {
    /// Gibt den festen Provider-Namen `"mock"` zurück.
    fn provider_name(&self) -> &str {
        "mock"
    }

    /// Asynchrone Embedding-Erzeugung für einen Eingabetext.
    ///
    /// **Ablauf:**
    /// Liefert sofort ein `Ok` mit dem geklonten `fixed_output` oder
    /// einem mit `0.1f32` gefüllten Vektor der Dimension `dim`.
    fn embed<'a>(&'a self, _text: &'a str) -> BoxFuture<'a, Result<Vec<f32>, EmbeddingError>> {
        Box::pin(async move {
            Ok(self
                .fixed_output
                .clone()
                .unwrap_or_else(|| vec![0.1f32; self.dim]))
        })
    }

    /// Liefert die konfigurierte Vektordimension.
    fn embedding_dim(&self) -> usize {
        self.dim
    }
}

// ANCHOR[contextra-testkit:METRIC-EVENT-ENUM-001]
/// Ein einzelnes aufgezeichnetes Metrik-Ereignis aus [`TestMetricsSink`].
///
/// **Zweck:** Dient in Test-Assertions der genauen Überprüfung, ob Instrumentierungen
/// die richtigen Namen, Werte und Labels gemeldet haben.
#[derive(Debug, Clone, PartialEq)]
pub enum MetricEvent {
    /// Inkrement-Ereignis eines monotonen Zählers (Counter).
    Counter {
        /// Name der Metrik.
        name: String,
        /// Um wie viel der Zähler erhöht wurde.
        value: u64,
        /// Schlüssel-Wert-Labels (Tags) zur Dimensionierung.
        labels: Vec<(String, String)>,
    },
    /// Absoluter Füllstandsmesser (Gauge).
    Gauge {
        /// Name der Metrik.
        name: String,
        /// Aktueller Wert.
        value: f64,
        /// Schlüssel-Wert-Labels.
        labels: Vec<(String, String)>,
    },
    /// Beobachtung einer Wertverteilung (Histogramm, z. B. Latenz).
    Histogram {
        /// Name der Metrik.
        name: String,
        /// Gemessener Wert.
        value: f64,
        /// Schlüssel-Wert-Labels.
        labels: Vec<(String, String)>,
    },
}

// ANCHOR[contextra-testkit:TEST-METRICS-SINK-STRUCT-001]
/// In-Memory-Implementierung von [`contextra_ports::MetricsSink`] für Tests.
///
/// **Zweck:** Fängt alle Metrik-Aufrufe ab und speichert sie chronologisch
/// in einer threadsicheren Liste zur späteren Auswertung in Unit-Tests.
#[derive(Debug, Default, Clone)]
pub struct TestMetricsSink {
    /// Mutex-geschützte Liste aller eingegangenen Ereignisse.
    events: std::sync::Arc<std::sync::Mutex<Vec<MetricEvent>>>,
}

impl TestMetricsSink {
    // ANCHOR[contextra-testkit:TEST-METRICS-SINK-NEW-001]
    /// Erstellt eine leere [`TestMetricsSink`].
    pub fn new() -> Self {
        Self::default()
    }

    // ANCHOR[contextra-testkit:TEST-METRICS-SINK-EVENTS-001]
    /// Erstellt einen Schnappschuss (Snapshot) aller bisher aufgezeichneten Ereignisse.
    ///
    /// Behandelt vergiftete Mutexe (`PoisonError`) defensiv via `into_inner()`.
    pub fn events(&self) -> Vec<MetricEvent> {
        self.events
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }

    // ANCHOR[contextra-testkit:TEST-METRICS-SINK-CLEAR-001]
    /// Leert die Liste aller aufgezeichneten Metrik-Ereignisse.
    pub fn clear(&self) {
        self.events
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clear();
    }
}

// ANCHOR[contextra-testkit:TEST-METRICS-SINK-TRAIT-IMPL-001]
impl MetricsSink for TestMetricsSink {
    /// Zeichnet ein Zähler-Ereignis (Counter) auf.
    fn record_counter(&self, name: &str, value: u64, labels: &[(&str, &str)]) {
        let labels_vec = labels
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        self.events
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .push(MetricEvent::Counter {
                name: name.to_string(),
                value,
                labels: labels_vec,
            });
    }

    /// Zeichnet einen absoluten Messwert (Gauge) auf.
    fn record_gauge(&self, name: &str, value: f64, labels: &[(&str, &str)]) {
        let labels_vec = labels
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        self.events
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .push(MetricEvent::Gauge {
                name: name.to_string(),
                value,
                labels: labels_vec,
            });
    }

    /// Zeichnet eine Histogramm-Beobachtung auf.
    fn record_histogram(&self, name: &str, value: f64, labels: &[(&str, &str)]) {
        let labels_vec = labels
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        self.events
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .push(MetricEvent::Histogram {
                name: name.to_string(),
                value,
                labels: labels_vec,
            });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use contextra_ports::TextEmbeddingEngine;

    // ANCHOR[contextra-testkit:TEST-MOCKS-EMBEDDER-001]
    // Prüft Einbettungs-Dimension und deterministische Vektorerzeugung von MockEmbedder.
    #[tokio::test]
    async fn test_mock_embedder_methods() -> Result<(), Box<dyn std::error::Error>> {
        let embedder = MockEmbedder::new(4);
        assert_eq!(embedder.provider_name(), "mock");
        assert_eq!(embedder.embedding_dim(), 4);

        let vec = EmbeddingProvider::embed(&embedder, "test").await?;
        assert_eq!(vec, vec![0.1, 0.1, 0.1, 0.1]);

        let fixed = vec![1.0, 2.0, 3.0];
        let fixed_embedder = MockEmbedder::with_fixed_output(fixed.clone());
        assert_eq!(fixed_embedder.embedding_dim(), 3);
        let res = EmbeddingProvider::embed(&fixed_embedder, "hello").await?;
        assert_eq!(res, fixed);
        Ok(())
    }

    // ANCHOR[contextra-testkit:TEST-MOCKS-BATCH-001]
    // Prüft Batch-Verarbeitung und Blanket-Trait-Implementierung für TextEmbeddingEngine.
    #[tokio::test]
    async fn test_embedding_provider_batch_and_blanket_trait(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let embedder = MockEmbedder::new(2);
        let batch_res = EmbeddingProvider::embed_batch(&embedder, &["a", "b"]).await?;
        assert_eq!(batch_res.len(), 2);
        assert_eq!(batch_res.first(), Some(&vec![0.1, 0.1]));

        let engine: &dyn TextEmbeddingEngine = &embedder;
        let single = engine.embed("text").await?;
        assert_eq!(single, vec![0.1, 0.1]);

        let batch = engine.embed_batch(&["x", "y"]).await?;
        assert_eq!(batch.len(), 2);
        Ok(())
    }

    // ANCHOR[contextra-testkit:TEST-MOCKS-SINK-001]
    // Prüft die chronologische Aufzeichnung und das Leeren von Counter-, Gauge- und Histogramm-Ereignissen.
    #[test]
    fn test_test_metrics_sink() {
        let sink = TestMetricsSink::new();
        sink.record_counter("c1", 5, &[("env", "test")]);
        sink.record_gauge("g1", 1.5, &[]);
        sink.record_histogram("h1", 0.02, &[("op", "commit")]);

        let events = sink.events();
        assert_eq!(events.len(), 3);
        assert_eq!(
            events.first(),
            Some(&MetricEvent::Counter {
                name: "c1".to_string(),
                value: 5,
                labels: vec![("env".to_string(), "test".to_string())],
            })
        );
        assert_eq!(
            events.get(1),
            Some(&MetricEvent::Gauge {
                name: "g1".to_string(),
                value: 1.5,
                labels: vec![],
            })
        );
        assert_eq!(
            events.get(2),
            Some(&MetricEvent::Histogram {
                name: "h1".to_string(),
                value: 0.02,
                labels: vec![("op".to_string(), "commit".to_string())],
            })
        );

        sink.clear();
        assert!(sink.events().is_empty());
    }
}
