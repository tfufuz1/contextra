// FILE-CONTEXT
// STAND: 2026-09-15T16:10:40Z (SESSION: ec33599e)
// ZWECK: Haupt-Reranker-API (`CrossEncoderReranker`) und Backend-Abstraktion.

use contextra_core::{ConfigFingerprint, ContextraError};
use contextra_rank::PlattScaler;

use super::config::{MAX_CANDIDATES, RerankConfig, RerankResult};

#[cfg(feature = "onnx")]
use super::onnx::OnnxReranker;

/// Interne Backend-Varianten für den CrossEncoderReranker.
#[allow(clippy::large_enum_variant)]
enum RerankerBackend {
    /// Passthrough-Backend, falls das `onnx`-Feature deaktiviert ist.
    #[allow(dead_code)]
    Passthrough,
    /// Echtes Inferenz-Backend über ONNX Runtime.
    #[cfg(feature = "onnx")]
    Onnx(Box<OnnxReranker>),
}

/// Unified public `CrossEncoderReranker` struct independent of active feature flags.
pub struct CrossEncoderReranker {
    pub(super) _config: RerankConfig,
    backend: RerankerBackend,
    /// Online-Kalibrierungshistorie für Platt-Fitting.
    /// Tuple: (raw_logit, is_relevant: bool)
    pub(super) calibration_buffer: parking_lot::Mutex<Vec<(f32, bool)>>,
    /// Minimum Observationen vor erstem Fit (Default: 50, aus IsotonicCalibrator-Warmup)
    calibration_warmup: usize,
    /// Aktuell gefittetes Kalibrierungsmodell.
    fitted_calibration: parking_lot::RwLock<PlattScaler>,
    /// ConfigFingerprint für INV-CAL-2: Invalide bei Modellwechsel.
    calibration_fingerprint: parking_lot::RwLock<Option<ConfigFingerprint>>,
}

impl CrossEncoderReranker {
    /// Records implicit feedback: top `implicit_relevant_k` results are marked as relevant,
    /// the rest as not relevant.
    ///
    /// Call this AFTER every successful `rerank()` call to feed calibration.
    /// Thread-safe: uses internal Mutex/RwLock.
    pub fn record_implicit_feedback(&self, results: &[RerankResult], implicit_relevant_k: usize) {
        if matches!(self.backend, RerankerBackend::Passthrough) {
            return;
        }
        for r in results {
            self.record_outcome(r.score, r.original_index < implicit_relevant_k);
        }
    }

    /// Returns true if PlattScaler has been fitted (warmup completed).
    pub fn is_calibrated(&self) -> bool {
        !self.fitted_calibration.read().is_identity()
    }

    /// Returns number of observations recorded.
    pub fn calibration_observation_count(&self) -> usize {
        self.calibration_buffer.lock().len()
    }

    /// Nimmt Feedback-Signal auf (raw logit + Relevanz-Label).
    /// Triggert Re-Fit wenn Warmup erreicht.
    pub fn record_outcome(&self, raw_logit: f32, is_relevant: bool) {
        let mut buf = self.calibration_buffer.lock();
        buf.push((raw_logit, is_relevant));
        if buf.len() >= self.calibration_warmup {
            let new_calib = PlattScaler::fit(&buf);
            *self.fitted_calibration.write() = new_calib;
        }
    }

    /// Appliziert aktuell gefittete Kalibrierung auf rohen Logit-Score.
    pub fn calibrate(&self, raw_logit: f32) -> f32 {
        self.fitted_calibration.read().apply(raw_logit)
    }

    /// Invalidiert Kalibrierung bei Modellwechsel (INV-CAL-2).
    pub fn invalidate_calibration(&self, new_fingerprint: ConfigFingerprint) {
        *self.fitted_calibration.write() = PlattScaler::identity();
        self.calibration_buffer.lock().clear();
        *self.calibration_fingerprint.write() = Some(new_fingerprint);
    }

    /// Gibt die aktuell aktiv gefittete Kalibrierung zurück.
    pub fn fitted_calibration(&self) -> PlattScaler {
        self.fitted_calibration.read().clone()
    }

    /// Erstellt einen Passthrough-CrossEncoderReranker (für Benchmarks/Tests ohne ONNX-Modelldatei).
    pub fn passthrough() -> Self {
        Self::passthrough_with_config(RerankConfig::default())
    }

    /// Erstellt einen Passthrough-CrossEncoderReranker mit einer benutzerdefinierten Konfiguration.
    pub fn passthrough_with_config(config: RerankConfig) -> Self {
        let calibration = config.calibration.clone();
        let warmup = config.calibration_warmup;
        Self {
            _config: config,
            backend: RerankerBackend::Passthrough,
            calibration_buffer: parking_lot::Mutex::new(Vec::new()),
            calibration_warmup: warmup,
            fitted_calibration: parking_lot::RwLock::new(calibration),
            calibration_fingerprint: parking_lot::RwLock::new(None),
        }
    }

    /// Returns a reference to the active `RerankConfig`.
    pub fn config(&self) -> &RerankConfig {
        &self._config
    }

    /// Setzt ein gefittetes `PlattScaler` Modell für den CrossEncoderReranker.
    pub fn with_calibration(mut self, calibration: PlattScaler) -> Self {
        self._config.calibration = calibration.clone();
        *self.fitted_calibration.write() = calibration;
        self
    }

    /// Erstellt einen neuen CrossEncoderReranker.
    pub fn new(config: RerankConfig) -> Result<Self, ContextraError> {
        let calibration = config.calibration.clone();
        let warmup = config.calibration_warmup;
        #[cfg(feature = "onnx")]
        {
            let onnx = OnnxReranker::new(config.clone())?;
            Ok(Self {
                _config: config,
                backend: RerankerBackend::Onnx(Box::new(onnx)),
                calibration_buffer: parking_lot::Mutex::new(Vec::new()),
                calibration_warmup: warmup,
                fitted_calibration: parking_lot::RwLock::new(calibration),
                calibration_fingerprint: parking_lot::RwLock::new(None),
            })
        }
        #[cfg(not(feature = "onnx"))]
        {
            Ok(Self {
                _config: config,
                backend: RerankerBackend::Passthrough,
                calibration_buffer: parking_lot::Mutex::new(Vec::new()),
                calibration_warmup: warmup,
                fitted_calibration: parking_lot::RwLock::new(calibration),
                calibration_fingerprint: parking_lot::RwLock::new(None),
            })
        }
    }

    /// Rerankt Kandidaten für eine Abfrage.
    pub async fn rerank(
        &self,
        _query: &str,
        candidates: &[String],
    ) -> Result<Vec<RerankResult>, ContextraError> {
        if candidates.len() > MAX_CANDIDATES {
            return Err(ContextraError::InvalidInput(format!(
                "Candidate batch size {} exceeds maximum allowed limit {}",
                candidates.len(),
                MAX_CANDIDATES
            )));
        }

        if let Some(delay_ms) = self._config.simulate_delay_ms {
            tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
        }

        match &self.backend {
            RerankerBackend::Passthrough => Ok(candidates
                .iter()
                .enumerate()
                .map(|(i, _)| RerankResult {
                    original_index: i,
                    score: 1.0 - (i as f32 * 0.01),
                })
                .collect()),
            #[cfg(feature = "onnx")]
            RerankerBackend::Onnx(onnx) => {
                let calib = self.fitted_calibration.read().clone();
                onnx.rerank(_query, candidates, &calib).await
            }
        }
    }
}
