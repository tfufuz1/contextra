// FILE-CONTEXT
// STAND: 2026-10-25
// ZWECK: Mock implementations for testing (MockEmbedder, TestMetricsSink, MetricEvent).
// INVARIANTEN: No unsafe code allowed in testkit (#![forbid(unsafe_code)]).

use contextra_ports::{BoxFuture, EmbeddingError, EmbeddingProvider, MetricsSink};

/// Deterministic mock embedding provider for tests.
pub struct MockEmbedder {
    /// Target embedding dimensionality.
    pub dim: usize,
    /// Fixed output vector for deterministic testing.
    pub fixed_output: Option<Vec<f32>>,
}

impl MockEmbedder {
    /// Creates a new `MockEmbedder` with the specified dimension.
    pub fn new(dim: usize) -> Self {
        Self {
            dim,
            fixed_output: None,
        }
    }

    /// Creates a new `MockEmbedder` with a fixed output vector.
    pub fn with_fixed_output(fixed_output: Vec<f32>) -> Self {
        let dim = fixed_output.len();
        Self {
            dim,
            fixed_output: Some(fixed_output),
        }
    }
}

impl EmbeddingProvider for MockEmbedder {
    fn provider_name(&self) -> &str {
        "mock"
    }

    fn embed<'a>(&'a self, _text: &'a str) -> BoxFuture<'a, Result<Vec<f32>, EmbeddingError>> {
        Box::pin(async move {
            Ok(self
                .fixed_output
                .clone()
                .unwrap_or_else(|| vec![0.1f32; self.dim]))
        })
    }

    fn embedding_dim(&self) -> usize {
        self.dim
    }
}

/// A recorded metric event captured by [`TestMetricsSink`].
#[derive(Debug, Clone, PartialEq)]
pub enum MetricEvent {
    /// Counter increment event.
    Counter {
        /// Metric name.
        name: String,
        /// Incremented value.
        value: u64,
        /// Metric key-value labels.
        labels: Vec<(String, String)>,
    },
    /// Gauge setting event.
    Gauge {
        /// Metric name.
        name: String,
        /// Absolute gauge value.
        value: f64,
        /// Metric key-value labels.
        labels: Vec<(String, String)>,
    },
    /// Histogram observation event.
    Histogram {
        /// Metric name.
        name: String,
        /// Observed value.
        value: f64,
        /// Metric key-value labels.
        labels: Vec<(String, String)>,
    },
}

/// An in-memory [`MetricsSink`] implementation for testing that records all emitted metric events.
#[derive(Debug, Default, Clone)]
pub struct TestMetricsSink {
    events: std::sync::Arc<std::sync::Mutex<Vec<MetricEvent>>>,
}

impl TestMetricsSink {
    /// Creates a new empty [`TestMetricsSink`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a snapshot copy of all recorded metric events.
    pub fn events(&self) -> Vec<MetricEvent> {
        self.events
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }

    /// Clears all recorded metric events.
    pub fn clear(&self) {
        self.events
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clear();
    }
}

impl MetricsSink for TestMetricsSink {
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

    #[tokio::test]
    async fn test_embedding_provider_batch_and_blanket_trait(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let embedder = MockEmbedder::new(2);
        let batch_res = EmbeddingProvider::embed_batch(&embedder, &["a", "b"]).await?;
        assert_eq!(batch_res.len(), 2);
        assert_eq!(batch_res[0], vec![0.1, 0.1]);

        let engine: &dyn TextEmbeddingEngine = &embedder;
        let single = engine.embed("text").await?;
        assert_eq!(single, vec![0.1, 0.1]);

        let batch = engine.embed_batch(&["x", "y"]).await?;
        assert_eq!(batch.len(), 2);
        Ok(())
    }

    #[test]
    fn test_test_metrics_sink() {
        let sink = TestMetricsSink::new();
        sink.record_counter("c1", 5, &[("env", "test")]);
        sink.record_gauge("g1", 1.5, &[]);
        sink.record_histogram("h1", 0.02, &[("op", "commit")]);

        let events = sink.events();
        assert_eq!(events.len(), 3);
        assert_eq!(
            events[0],
            MetricEvent::Counter {
                name: "c1".to_string(),
                value: 5,
                labels: vec![("env".to_string(), "test".to_string())],
            }
        );
        assert_eq!(
            events[1],
            MetricEvent::Gauge {
                name: "g1".to_string(),
                value: 1.5,
                labels: vec![],
            }
        );
        assert_eq!(
            events[2],
            MetricEvent::Histogram {
                name: "h1".to_string(),
                value: 0.02,
                labels: vec![("op".to_string(), "commit".to_string())],
            }
        );

        sink.clear();
        assert!(sink.events().is_empty());
    }
}
