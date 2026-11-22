// FILE-CONTEXT
// STAND: 2026-09-15T16:10:40Z (SESSION: ec33599e)
// ZWECK: Cross-Encoder Reranking Konfiguration und Ergebnisse.

use contextra_rank::PlattScaler;

/// Alias für `PlattScaler` zur Rückwärtskompatibilität und ADR-070-Konformität.
pub use contextra_rank::PlattScaler as PlattScaledSigmoid;

/// Maximale Anzahl von Kandidaten pro Reranking-Aufruf zur Vermeidung unbegrenzter Allokationen.
pub const MAX_CANDIDATES: usize = 10_000;

/// Re-export `RerankResult` from `contextra_types` for backward compatibility.
pub use contextra_types::RerankResult;

use contextra_types::ContextraError;
use std::path::{Path, PathBuf};

/// Konfiguration für Cross-Encoder Reranking.
#[derive(Debug, Clone)]
pub struct RerankConfig {
    /// Pfad zur ONNX-Modelldatei (bge-reranker-base.onnx)
    pub model_path: PathBuf,
    /// Pfad zur Tokenizer-Konfigurationsdatei (tokenizer.json)
    pub tokenizer_path: PathBuf,
    /// Maximale Tokenlänge für (query, candidate) Pair
    pub max_length: usize,
    /// Batch-Größe für parallele Inferenz
    pub batch_size: usize,
    /// Intra-Op Threads für ONNX Runtime Execution
    pub intra_op_threads: usize,
    /// Inter-Op Threads für ONNX Runtime Execution
    pub inter_op_threads: usize,
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
            model_path: PathBuf::from("models/bge-reranker-base.onnx"),
            tokenizer_path: PathBuf::from("models/tokenizer.json"),
            max_length: 512,
            batch_size: 8,
            intra_op_threads: 1,
            inter_op_threads: 1,
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

    /// Validiert die Konfigurationsparameter vor der Modellinitialisierung.
    pub fn validate(&self) -> Result<(), ContextraError> {
        if !self.model_path.exists() {
            return Err(ContextraError::InvalidInput(format!(
                "ONNX model file not found at {:?}",
                self.model_path
            )));
        }
        if !self.tokenizer_path.exists() {
            return Err(ContextraError::InvalidInput(format!(
                "Tokenizer file not found at {:?}",
                self.tokenizer_path
            )));
        }
        if self.batch_size == 0 || self.batch_size > MAX_CANDIDATES {
            return Err(ContextraError::InvalidInput(format!(
                "batch_size must be > 0 and <= {MAX_CANDIDATES}, got {}",
                self.batch_size
            )));
        }
        if self.max_length == 0 {
            return Err(ContextraError::InvalidInput(
                "max_length must be > 0".to_string(),
            ));
        }
        if self.intra_op_threads == 0 {
            return Err(ContextraError::InvalidInput(
                "intra_op_threads must be > 0".to_string(),
            ));
        }
        if self.inter_op_threads == 0 {
            return Err(ContextraError::InvalidInput(
                "inter_op_threads must be > 0".to_string(),
            ));
        }
        Ok(())
    }
}

/// Standalone production configuration for ONNX CrossEncoder Reranker.
#[derive(Debug, Clone)]
pub struct OnnxConfig {
    pub inner: RerankConfig,
}

impl OnnxConfig {
    /// Creates an `OnnxConfig` with required paths and default settings.
    pub fn new(model_path: impl AsRef<Path>, tokenizer_path: impl AsRef<Path>) -> Self {
        Self {
            inner: RerankConfig {
                model_path: model_path.as_ref().to_path_buf(),
                tokenizer_path: tokenizer_path.as_ref().to_path_buf(),
                ..RerankConfig::default()
            },
        }
    }

    /// Validates configuration parameters and file existence.
    pub fn validate(&self) -> Result<(), ContextraError> {
        self.inner.validate()
    }
}

/// Builder for `CrossEncoderReranker` instances using ONNX models.
///
/// Ensures offline-first behavior and performs pre-validation before loading ONNX Runtime models.
///
/// # Example
/// ```no_run
/// use contextra_infer_onnx::reranker::{OnnxRerankerBuilder, CrossEncoderReranker};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let reranker = OnnxRerankerBuilder::new("models/bge-reranker.onnx", "models/tokenizer.json")
///     .batch_size(16)
///     .max_sequence_length(256)
///     .intra_op_threads(2)
///     .build()?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct OnnxRerankerBuilder {
    config: RerankConfig,
}

impl OnnxRerankerBuilder {
    /// Creates a builder with the required model and tokenizer paths.
    pub fn new(model_path: impl AsRef<Path>, tokenizer_path: impl AsRef<Path>) -> Self {
        Self {
            config: RerankConfig {
                model_path: model_path.as_ref().to_path_buf(),
                tokenizer_path: tokenizer_path.as_ref().to_path_buf(),
                ..RerankConfig::default()
            },
        }
    }

    /// Sets the batch size for parallel inference.
    pub fn batch_size(mut self, batch_size: usize) -> Self {
        self.config.batch_size = batch_size;
        self
    }

    /// Sets maximum sequence token length for (query, document) pairs.
    pub fn max_sequence_length(mut self, max_length: usize) -> Self {
        self.config.max_length = max_length;
        self
    }

    /// Sets number of intra-op threads for ONNX execution.
    pub fn intra_op_threads(mut self, threads: usize) -> Self {
        self.config.intra_op_threads = threads;
        self
    }

    /// Sets number of inter-op threads for ONNX execution.
    pub fn inter_op_threads(mut self, threads: usize) -> Self {
        self.config.inter_op_threads = threads;
        self
    }

    /// Sets PlattScaler calibration model for logit normalization.
    pub fn calibration(mut self, calibration: PlattScaler) -> Self {
        self.config.calibration = calibration;
        self
    }

    /// Sets minimum observations before calibration warmup triggers.
    pub fn calibration_warmup(mut self, warmup: usize) -> Self {
        self.config.calibration_warmup = warmup;
        self
    }

    /// Builds a `CrossEncoderReranker` instance after pre-validating all configuration parameters.
    pub fn build(self) -> Result<super::cross_encoder::CrossEncoderReranker, ContextraError> {
        super::cross_encoder::CrossEncoderReranker::from_config(OnnxConfig { inner: self.config })
    }
}
