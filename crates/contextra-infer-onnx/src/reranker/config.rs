// FILE-CONTEXT
// STAND: 2026-09-15T16:10:40Z (SESSION: ec33599e)
// ZWECK: Cross-Encoder Reranking Konfiguration und Ergebnisse.

use contextra_rank::PlattScaler;

/// Alias für `PlattScaler` zur Rückwärtskompatibilität und ADR-070-Konformität.
pub use contextra_rank::PlattScaler as PlattScaledSigmoid;

/// Maximale Anzahl von Kandidaten pro Reranking-Aufruf zur Vermeidung unbegrenzter Allokationen.
pub const MAX_CANDIDATES: usize = 10_000;

/// Ergebnis einer Reranking-Operation.
#[derive(Debug, Clone)]
pub struct RerankResult {
    /// Ursprünglicher Index im Kandidaten-Array
    pub original_index: usize,
    /// Cross-Encoder Relevanz-Score (höher = relevanter)
    pub score: f32,
}

/// Konfiguration für Cross-Encoder Reranking.
#[derive(Debug, Clone)]
pub struct RerankConfig {
    /// Pfad zur ONNX-Modelldatei (bge-reranker-base.onnx)
    pub model_path: std::path::PathBuf,
    /// Pfad zur Tokenizer-Konfigurationsdatei (tokenizer.json)
    pub tokenizer_path: std::path::PathBuf,
    /// Maximale Tokenlänge für (query, candidate) Pair
    pub max_length: usize,
    /// Batch-Größe für parallele Inferenz
    pub batch_size: usize,
    /// Optionale Platt-Scaling Kalibrierung für Roh-Logits
    pub calibration: PlattScaler,
    /// Minimum Observationen vor erstem Fit (Default: 50, aus IsotonicCalibrator-Warmup)
    pub calibration_warmup: usize,
    /// Maximum allowed execution time in milliseconds before timing out. Default: 500ms.
    pub rerank_deadline_ms: Option<u64>,
    /// Simulated delay in milliseconds for testing timeouts. Default: None.
    pub simulate_delay_ms: Option<u64>,
}

impl Default for RerankConfig {
    fn default() -> Self {
        Self {
            model_path: std::path::PathBuf::from("models/bge-reranker-base.onnx"),
            tokenizer_path: std::path::PathBuf::from("models/tokenizer.json"),
            max_length: 512,
            batch_size: 8,
            calibration: PlattScaler::identity(),
            calibration_warmup: 50,
            rerank_deadline_ms: Some(500),
            simulate_delay_ms: None,
        }
    }
}

impl RerankConfig {
    /// Setzt ein gefittetes `PlattScaler` Modell für die Logit-Kalibrierung.
    pub fn with_calibration(mut self, calibration: PlattScaler) -> Self {
        self.calibration = calibration;
        self
    }
}
