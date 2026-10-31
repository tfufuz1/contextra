// FILE-CONTEXT
// STAND: 2026-10-31
// ZWECK: Haupt-Reranker-API (`CrossEncoderReranker`) und Backend-Abstraktion (Ring 2).
// INVARIANTEN: Falls onnx-Feature inaktiv, greift transparenter Passthrough-Fallback.
// CONCURRENCY: parking_lot Mutex/RwLock für kalibrierte Online-Updates.

//! # Cross-Encoder Reranker Hauptkomponente
//!
//! Stellt die zentrale Abstraktion [`CrossEncoderReranker`] bereit.
//! Abhängig vom Feature-Flag `onnx` schaltet der Reranker transparent zwischen:
//! 1. Echtem Inferenz-Backend über ONNX Runtime ([`super::onnx::OnnxReranker`]).
//! 2. Passthrough-Backend (keine externen Abhängigkeiten, reproduzierbare Fallback-Sortierung).
//!
//! Beinhaltet zudem eine adaptive Online-Kalibrierung per Platt-Scaling (`PlattScaler`),
//! um kontinuierlich implizites Feedback (Klick-/Relevanzsignale) zu lernen.

use contextra_ports::{BoxFuture, Reranker};
use contextra_rank::PlattScaler;
use contextra_types::{ConfigFingerprint, ContextraError, RerankResult};

use super::config::{OnnxConfig, OnnxRerankerBuilder, RerankConfig, MAX_CANDIDATES};
use std::path::Path;

#[cfg(feature = "onnx")]
use super::onnx::OnnxReranker;

// ANCHOR[contextra-infer-onnx:RERANKER-BACKEND-ENUM]
/// Interne Backend-Varianten für den CrossEncoderReranker.
#[allow(clippy::large_enum_variant)]
enum RerankerBackend {
    /// Passthrough-Backend, falls das `onnx`-Feature deaktiviert ist.
    Passthrough,
    /// Echtes Inferenz-Backend über ONNX Runtime.
    #[cfg(feature = "onnx")]
    Onnx(Box<OnnxReranker>),
}

// ANCHOR[contextra-infer-onnx:CROSS-ENCODER-STRUCT]
/// Einheitlicher, öffentlicher `CrossEncoderReranker`-Typ, unabhängig von aktiven Feature-Flags.
///
/// Bietet eine thread-sichere Schnittstelle für die semantische Nachbewertung von Kandidatenlisten.
/// Unterstützt Online-Kalibrierung (`PlattScaler`), um Roh-Logits in Wahrscheinlichkeiten zu überführen.
pub struct CrossEncoderReranker {
    /// Aktive Konfiguration des Rerankers.
    pub(super) _config: RerankConfig,
    /// Internes Backend (Passthrough oder ONNX).
    backend: RerankerBackend,
    /// Online-Kalibrierungshistorie für Platt-Fitting.
    /// Tupel: `(raw_logit, is_relevant: bool)`.
    pub(super) calibration_buffer: parking_lot::Mutex<Vec<(f32, bool)>>,
    /// Minimum Observationen vor erstem Fit (Default: 50, aus IsotonicCalibrator-Warmup).
    calibration_warmup: usize,
    /// Aktuell gefittetes Kalibrierungsmodell (lese-optimiert durch `RwLock`).
    fitted_calibration: parking_lot::RwLock<PlattScaler>,
    /// ConfigFingerprint für Invariante INV-CAL-2: Invalide bei Modellwechsel.
    calibration_fingerprint: parking_lot::RwLock<Option<ConfigFingerprint>>,
}

impl CrossEncoderReranker {
    // ANCHOR[contextra-infer-onnx:RECORD-IMPLICIT-FEEDBACK]
    /// Zeichnet implizites Benutzer-Feedback auf: Die Top `implicit_relevant_k` Dokumente
    /// werden als relevant (`true`) markiert, die restlichen Dokumente als irrelevant (`false`).
    ///
    /// Sollte nach erfolgreichem `rerank()` aufgerufen werden, um die Online-Kalibrierung zu speisen.
    /// Thread-safe: nutzt interne Mutex- und RwLock-Sperren.
    ///
    /// # Ablauf
    /// 1. Prüfen, ob Backend Passthrough ist (falls ja, keine Kalibrierung nötig).
    /// 2. Für jedes Ergebnis `record_outcome` mit Score und Relevanz-Label aufrufen.
    pub fn record_implicit_feedback(&self, results: &[RerankResult], implicit_relevant_k: usize) {
        if matches!(self.backend, RerankerBackend::Passthrough) {
            return;
        }
        for r in results {
            self.record_outcome(r.score, r.original_index < implicit_relevant_k);
        }
    }

    // ANCHOR[contextra-infer-onnx:IS-CALIBRATED]
    /// Gibt `true` zurück, wenn der `PlattScaler` gefittet wurde (Warmup abgeschlossen).
    pub fn is_calibrated(&self) -> bool {
        !self.fitted_calibration.read().is_identity()
    }

    // ANCHOR[contextra-infer-onnx:CALIBRATION-OBSERVATIONS]
    /// Gibt die Gesamtanzahl der bisher gesammelten Kalibrierungs-Beobachtungen zurück.
    pub fn calibration_observation_count(&self) -> usize {
        self.calibration_buffer.lock().len()
    }

    // ANCHOR[contextra-infer-onnx:RECORD-OUTCOME]
    /// Nimmt ein Feedback-Signal auf (Roh-Logit + Relevanz-Label).
    /// Triggert automatisch einen Re-Fit des PlattScalers, sobald die Warmup-Schwelle erreicht ist.
    ///
    /// # Ablauf
    /// 1. `calibration_buffer` sperren und Tupel `(raw_logit, is_relevant)` anhängen.
    /// 2. Falls Puffergröße >= `calibration_warmup`: `PlattScaler::fit(&buf)` berechnen.
    /// 3. Gefittetes Modell atomar in `fitted_calibration` über RwLock schreiben.
    pub fn record_outcome(&self, raw_logit: f32, is_relevant: bool) {
        let mut buf = self.calibration_buffer.lock();
        buf.push((raw_logit, is_relevant));
        if buf.len() >= self.calibration_warmup {
            let new_calib = PlattScaler::fit(&buf);
            *self.fitted_calibration.write() = new_calib;
        }
    }

    // ANCHOR[contextra-infer-onnx:CALIBRATE-METHOD]
    /// Appliziert die aktuell gefittete Kalibrierung auf einen rohen Logit-Score.
    pub fn calibrate(&self, raw_logit: f32) -> f32 {
        self.fitted_calibration.read().apply(raw_logit)
    }

    // ANCHOR[contextra-infer-onnx:INVALIDATE-CALIBRATION]
    /// Invalidiert die Kalibrierung bei einem Modellwechsel gemäß Invariante INV-CAL-2.
    ///
    /// Setzt den `PlattScaler` auf Identität zurück, leert den Puffer und setzt den neuen Fingerprint.
    pub fn invalidate_calibration(&self, new_fingerprint: ConfigFingerprint) {
        *self.fitted_calibration.write() = PlattScaler::identity();
        self.calibration_buffer.lock().clear();
        *self.calibration_fingerprint.write() = Some(new_fingerprint);
    }

    // ANCHOR[contextra-infer-onnx:FITTED-CALIBRATION]
    /// Gibt einen Klon des aktuell aktiven Kalibrierungsmodells zurück.
    pub fn fitted_calibration(&self) -> PlattScaler {
        self.fitted_calibration.read().clone()
    }

    // ANCHOR[contextra-infer-onnx:PASSTHROUGH-CONSTRUCTOR]
    /// Erstellt einen Passthrough-CrossEncoderReranker mit Standardkonfiguration (für Tests und Benchmarks).
    pub fn passthrough() -> Self {
        Self::passthrough_with_config(RerankConfig::default())
    }

    // ANCHOR[contextra-infer-onnx:PASSTHROUGH-WITH-CONFIG]
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

    // ANCHOR[contextra-infer-onnx:CONFIG-GETTER]
    /// Liefert eine Referenz auf die aktive [`RerankConfig`].
    pub fn config(&self) -> &RerankConfig {
        &self._config
    }

    // ANCHOR[contextra-infer-onnx:WITH-CALIBRATION-METHOD]
    /// Setzt ein initial gefittetes `PlattScaler`-Modell für den Reranker.
    pub fn with_calibration(mut self, calibration: PlattScaler) -> Self {
        self._config.calibration = calibration.clone();
        *self.fitted_calibration.write() = calibration;
        self
    }

    // ANCHOR[contextra-infer-onnx:BUILDER-FACTORY]
    /// Erstellt einen neuen Builder [`OnnxRerankerBuilder`].
    pub fn builder(
        model_path: impl AsRef<Path>,
        tokenizer_path: impl AsRef<Path>,
    ) -> OnnxRerankerBuilder {
        OnnxRerankerBuilder::new(model_path, tokenizer_path)
    }

    // ANCHOR[contextra-infer-onnx:FROM-CONFIG-METHOD]
    /// Erstellt einen `CrossEncoderReranker` aus einer [`OnnxConfig`].
    /// Führt vorab eine Validierung der Parameter und Dateipfade durch.
    pub fn from_config(config: OnnxConfig) -> Result<Self, ContextraError> {
        config.validate()?;
        Self::new(config.inner)
    }

    // ANCHOR[contextra-infer-onnx:NEW-RERANKER]
    /// Erstellt einen neuen `CrossEncoderReranker`.
    ///
    /// Falls `feature = "onnx"` aktiv ist, wird das ONNX-Backend initialisiert.
    /// Andernfalls wird der transparente Passthrough-Modus verwendet.
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

    // ANCHOR[contextra-infer-onnx:RERANK-METHOD]
    /// Führt das Reranking einer Kandidatenliste für eine Suchabfrage aus.
    ///
    /// # Ablauf
    /// 1. Prüfen, ob `candidates.len() <= MAX_CANDIDATES` ist (DoS-Schutz).
    /// 2. Falls `simulate_delay_ms` konfiguriert ist: Asynchron schlafen (Timeout-Simulation).
    /// 3. Im Passthrough-Modus: Deterministische Scores absteigend vom Index berechnen.
    /// 4. Im ONNX-Modus: An `OnnxReranker::rerank` mit der aktuellen Kalibrierung delegieren.
    ///
    /// # Fehlerfälle
    /// - [`ContextraError::InvalidInput`]: Kandidaten-Batch überschreitet `MAX_CANDIDATES`.
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
                .map(|(i, _)| {
                    let score = 1.0 - (i as f32 * 0.01);
                    RerankResult {
                        index: i,
                        original_index: i,
                        score,
                        calibrated_score: score,
                    }
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

// ANCHOR[contextra-infer-onnx:RERANKER-TRAIT-IMPL]
/// Implementierung des systemweiten [`Reranker`]-Traits aus `contextra-ports`.
impl Reranker for CrossEncoderReranker {
    fn rerank<'a>(
        &'a self,
        query: &'a str,
        candidates: &'a [String],
    ) -> BoxFuture<'a, Result<Vec<RerankResult>, ContextraError>> {
        Box::pin(self.rerank(query, candidates))
    }

    fn record_implicit_feedback(&self, results: &[RerankResult], implicit_relevant_k: usize) {
        self.record_implicit_feedback(results, implicit_relevant_k);
    }

    fn rerank_deadline_ms(&self) -> Option<u64> {
        self._config.rerank_deadline_ms
    }
}
