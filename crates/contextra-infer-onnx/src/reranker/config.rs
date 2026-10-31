// FILE-CONTEXT
// STAND: 2026-10-31
// ZWECK: Cross-Encoder Reranking Konfiguration und Ergebnisse (Ring 2).
// INVARIANTEN: Batchgröße <= MAX_CANDIDATES zur Vermeidung unbegrenzter Speicherallokation.
// SIEHE AUCH: crates/contextra-infer-onnx/AGENTS.md, ADR-070

//! # Konfiguration für Cross-Encoder Reranker
//!
//! Dieses Modul enthält alle Typen und Builder zur Parametrisierung des Cross-Encoders.
//! Es definiert Grenzwerte für Speicherverbrauch und Batching, Dateipfade zu ONNX-Modellen
//! sowie Schnittstellen für die Platt-Scaling-Kalibrierung.

use contextra_rank::PlattScaler;

// ANCHOR[contextra-infer-onnx:PLATT-SCALED-SIGMOID-ALIAS]
/// Alias für [`PlattScaler`] zur Rückwärtskompatibilität und ADR-070-Konformität.
///
/// Platt-Scaling kalibriert die Roh-Logits (unnormierte Netzwerkausgaben) eines Cross-Encoders
/// über eine logistische Regression in echte Relevanz-Wahrscheinlichkeiten im Intervall `[0.0, 1.0]`.
pub use contextra_rank::PlattScaler as PlattScaledSigmoid;

// ANCHOR[contextra-infer-onnx:MAX-CANDIDATES-CONST]
/// Maximale Anzahl von Kandidaten pro Reranking-Aufruf (10.000).
///
/// Dient als Obergrenze zur Vermeidung unbegrenzter Speicherallokationen und CPU-Blockaden
/// bei fehlerhaften oder böswilligen Anfragen (Denial-of-Service-Schutz).
pub const MAX_CANDIDATES: usize = 10_000;

// ANCHOR[contextra-infer-onnx:RERANK-RESULT-REEXPORT]
/// Re-export `RerankResult` from `contextra_types` for backward compatibility.
///
/// Enthält den Kandidatenindex, den ursprünglichen Eingabeindex, den Rohscore sowie den kalibrierten Score.
pub use contextra_types::RerankResult;

use contextra_types::ContextraError;
use std::path::{Path, PathBuf};

// ANCHOR[contextra-infer-onnx:RERANK-CONFIG-STRUCT]
/// Vollständige Konfiguration für das Cross-Encoder Reranking.
///
/// Definiert Modellpfade, Thread-Einstellungen für die ONNX Runtime, Batchgrößen und
/// Timeouts für Inferenz-Deadlines.
#[derive(Debug, Clone)]
pub struct RerankConfig {
    /// Dateipfad zur ONNX-Modelldatei (z. B. `models/bge-reranker-base.onnx`).
    pub model_path: PathBuf,
    /// Dateipfad zur Tokenizer-Konfiguration (z. B. `models/tokenizer.json`).
    pub tokenizer_path: PathBuf,
    /// Maximale Tokenlänge für das zusammengefügte `(Query, Candidate)`-Paar (Standard: 512).
    pub max_length: usize,
    /// Batch-Größe für parallele Inferenz im ONNX Runtime Provider (Standard: 8).
    pub batch_size: usize,
    /// Anzahl der Intra-Op-Threads für die Ausführung einzelner Operatoren in ONNX Runtime (Standard: 1).
    pub intra_op_threads: usize,
    /// Anzahl der Inter-Op-Threads für parallele Operatorenausführung in ONNX Runtime (Standard: 1).
    pub inter_op_threads: usize,
    /// Optionale Platt-Scaling Kalibrierung zur Normalisierung von Roh-Logits in Wahrscheinlichkeiten.
    pub calibration: PlattScaler,
    /// Mindestanzahl beobachteter Interaktionssignale vor dem ersten Platt-Scaling-Fit (Standard: 50).
    pub calibration_warmup: usize,
    /// Maximale Ausführungszeit in Millisekunden vor einem Timeout (Standard: Some(500ms)).
    pub rerank_deadline_ms: Option<u64>,
    /// Künstliche Verzögerung in Millisekunden für Testzwecke von Deadlines (Standard: None).
    pub simulate_delay_ms: Option<u64>,
}

impl Default for RerankConfig {
    /// Erstellt eine Standardkonfiguration für lokale Modelle:
    /// - `model_path`: `models/bge-reranker-base.onnx`
    /// - `tokenizer_path`: `models/tokenizer.json`
    /// - `max_length`: 512 Tokens
    /// - `batch_size`: 8
    /// - `intra_op_threads` & `inter_op_threads`: jeweils 1
    /// - `calibration`: Identity (keine Skalierung)
    /// - `calibration_warmup`: 50
    /// - `rerank_deadline_ms`: 500 ms
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
    // ANCHOR[contextra-infer-onnx:RERANK-CONFIG-WITH-CALIBRATION]
    /// Konfiguriert ein vorab gefittetes [`PlattScaler`]-Modell für die Logit-Kalibrierung.
    pub fn with_calibration(mut self, calibration: PlattScaler) -> Self {
        self.calibration = calibration;
        self
    }

    // ANCHOR[contextra-infer-onnx:RERANK-CONFIG-VALIDATE]
    /// Validiert die Konfigurationsparameter und prüft die Dateiexistenz vor der Initialisierung.
    ///
    /// # Validierungsprüfungen
    /// 1. `model_path` muss auf eine existierende Datei verweisen.
    /// 2. `tokenizer_path` muss auf eine existierende Datei verweisen.
    /// 3. `batch_size` muss `> 0` und `<= MAX_CANDIDATES` sein.
    /// 4. `max_length` muss `> 0` sein.
    /// 5. `intra_op_threads` und `inter_op_threads` müssen `> 0` sein.
    ///
    /// # Fehler
    /// Gibt [`ContextraError::InvalidInput`] zurück, wenn eine der Bedingungen verletzt ist.
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

// ANCHOR[contextra-infer-onnx:ONNX-CONFIG-STRUCT]
/// Eigenständige Produktionskonfiguration für den ONNX-Cross-Encoder-Reranker.
/// Kapselt eine [`RerankConfig`].
#[derive(Debug, Clone)]
pub struct OnnxConfig {
    /// Die eingebettete Reranker-Konfiguration.
    pub inner: RerankConfig,
}

impl OnnxConfig {
    // ANCHOR[contextra-infer-onnx:ONNX-CONFIG-NEW]
    /// Erstellt eine `OnnxConfig` mit den erforderlichen Modell- und Tokenizerpfaden.
    /// Alle anderen Optionen werden mit Standardwerten initialisiert.
    pub fn new(model_path: impl AsRef<Path>, tokenizer_path: impl AsRef<Path>) -> Self {
        Self {
            inner: RerankConfig {
                model_path: model_path.as_ref().to_path_buf(),
                tokenizer_path: tokenizer_path.as_ref().to_path_buf(),
                ..RerankConfig::default()
            },
        }
    }

    // ANCHOR[contextra-infer-onnx:ONNX-CONFIG-VALIDATE]
    /// Validiert die Konfigurationsparameter und prüft die Dateiexistenz (delegiert an `inner.validate()`).
    pub fn validate(&self) -> Result<(), ContextraError> {
        self.inner.validate()
    }
}

// ANCHOR[contextra-infer-onnx:ONNX-RERANKER-BUILDER]
/// Builder zur schrittweisen Konstruktion von `CrossEncoderReranker`-Instanzen.
///
/// Erzwingt Offline-First-Betrieb und validiert vor dem Laden der ONNX-Session alle Parameter.
///
/// # Beispiel
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
    // ANCHOR[contextra-infer-onnx:BUILDER-NEW]
    /// Erstellt einen neuen Builder mit den erforderlichen Dateipfaden zu Modell und Tokenizer.
    pub fn new(model_path: impl AsRef<Path>, tokenizer_path: impl AsRef<Path>) -> Self {
        Self {
            config: RerankConfig {
                model_path: model_path.as_ref().to_path_buf(),
                tokenizer_path: tokenizer_path.as_ref().to_path_buf(),
                ..RerankConfig::default()
            },
        }
    }

    // ANCHOR[contextra-infer-onnx:BUILDER-BATCH-SIZE]
    /// Konfiguriert die Batch-Größe für parallele Inferenz (Standard: 8).
    pub fn batch_size(mut self, batch_size: usize) -> Self {
        self.config.batch_size = batch_size;
        self
    }

    // ANCHOR[contextra-infer-onnx:BUILDER-MAX-SEQ-LEN]
    /// Konfiguriert die maximale Token-Länge für `(Query, Candidate)`-Paare (Standard: 512).
    pub fn max_sequence_length(mut self, max_length: usize) -> Self {
        self.config.max_length = max_length;
        self
    }

    // ANCHOR[contextra-infer-onnx:BUILDER-INTRA-OP]
    /// Konfiguriert die Anzahl der Intra-Op-Threads in der ONNX Runtime.
    pub fn intra_op_threads(mut self, threads: usize) -> Self {
        self.config.intra_op_threads = threads;
        self
    }

    // ANCHOR[contextra-infer-onnx:BUILDER-INTER-OP]
    /// Konfiguriert die Anzahl der Inter-Op-Threads in der ONNX Runtime.
    pub fn inter_op_threads(mut self, threads: usize) -> Self {
        self.config.inter_op_threads = threads;
        self
    }

    // ANCHOR[contextra-infer-onnx:BUILDER-CALIBRATION]
    /// Übergibt ein kalibriertes `PlattScaler`-Modell für Wahrscheinlichkeits-Normalisierung.
    pub fn calibration(mut self, calibration: PlattScaler) -> Self {
        self.config.calibration = calibration;
        self
    }

    // ANCHOR[contextra-infer-onnx:BUILDER-WARMUP]
    /// Konfiguriert die Warmup-Schwelle (Mindestanzahl Beobachtungen vor erstem Online-Fit).
    pub fn calibration_warmup(mut self, warmup: usize) -> Self {
        self.config.calibration_warmup = warmup;
        self
    }

    // ANCHOR[contextra-infer-onnx:BUILDER-BUILD]
    /// Erzeugt die finale [`super::cross_encoder::CrossEncoderReranker`]-Instanz nach Validierung.
    ///
    /// # Ablauf
    /// 1. `config.validate()` ausführen.
    /// 2. Bei Erfolg `CrossEncoderReranker::new(config)` instanziieren.
    pub fn build(self) -> Result<super::cross_encoder::CrossEncoderReranker, ContextraError> {
        super::cross_encoder::CrossEncoderReranker::from_config(OnnxConfig { inner: self.config })
    }
}
