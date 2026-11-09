//! Cross-Encoder Reranker für In-Process Inferenz.
//!
//! Bewertet Relevanz von (Query, Dokument)-Paaren über ein Cross-Encoder-Modell.

use crate::config::RerankerConfig;
use crate::error::{Error, Result};
use crate::traits::Reranker;

#[cfg(feature = "onnx")]
use ort::{session::Session, value::Value};
#[cfg(feature = "onnx")]
use parking_lot::Mutex;
#[cfg(feature = "onnx")]
use std::sync::Arc;
#[cfg(feature = "onnx")]
use tokenizers::Tokenizer;

/// ONNX-basierter Cross-Encoder Reranker.
pub struct OnnxReranker {
    config: RerankerConfig,
    #[cfg(feature = "onnx")]
    tokenizer: Tokenizer,
    #[cfg(feature = "onnx")]
    session: Arc<Mutex<Session>>,
}

impl OnnxReranker {
    /// Erstellt eine neue Instanz des `OnnxReranker`.
    ///
    /// # Errors
    /// Gibt `Error` zurück, falls Konfigurationsvalidierung, Hash-Prüfung oder Modellinitialisierung fehlschlagen.
    pub fn new(config: RerankerConfig) -> Result<Self> {
        config.validate()?;

        #[cfg(feature = "onnx")]
        {
            let tokenizer = Tokenizer::from_file(&config.tokenizer_path)
                .map_err(|e| Error::Tokenization(e.to_string()))?;

            let session = Session::builder()
                .map_err(|e| Error::Inference(e.to_string()))?
                .with_intra_threads(config.num_threads)
                .map_err(|e| Error::Inference(e.to_string()))?
                .commit_from_file(&config.model_path)
                .map_err(|e| Error::Inference(e.to_string()))?;

            Ok(Self {
                config,
                tokenizer,
                session: Arc::new(Mutex::new(session)),
            })
        }

        #[cfg(not(feature = "onnx"))]
        {
            Err(Error::Unsupported(
                "ONNX-Feature ist nicht aktiviert in diesem Build".into(),
            ))
        }
    }

    /// Gibt eine Referenz auf die Konfiguration zurück.
    #[must_use]
    pub fn config(&self) -> &RerankerConfig {
        &self.config
    }
}

impl Reranker for OnnxReranker {
    fn score(&self, query: &str, docs: &[&str]) -> Result<Vec<f32>> {
        #[cfg(feature = "onnx")]
        {
            if docs.is_empty() {
                return Ok(Vec::new());
            }

            let batch_size = docs.len();
            let mut pairs = Vec::with_capacity(batch_size);
            for &d in docs {
                pairs.push((query, d));
            }

            let encodings = self
                .tokenizer
                .encode_batch(pairs, true)
                .map_err(|e| Error::Tokenization(e.to_string()))?;

            let seq_len = encodings
                .first()
                .map_or(0, |enc| enc.get_ids().len())
                .min(self.config.max_length);

            if seq_len == 0 {
                return Ok(vec![0.0f32; batch_size]);
            }

            let mut input_ids = Vec::with_capacity(batch_size.saturating_mul(seq_len));
            let mut attention_mask = Vec::with_capacity(batch_size.saturating_mul(seq_len));

            for enc in &encodings {
                let ids = enc.get_ids();
                let mask = enc.get_attention_mask();

                for i in 0..seq_len {
                    let id_val = ids.get(i).copied().unwrap_or(0) as i64;
                    let mask_val = mask.get(i).copied().unwrap_or(0) as i64;
                    input_ids.push(id_val);
                    attention_mask.push(mask_val);
                }
            }

            let input_shape = [batch_size, seq_len];
            let input_ids_array =
                ndarray::Array2::from_shape_vec(input_shape, input_ids)
                    .map_err(|e| Error::Inference(e.to_string()))?;
            let attention_mask_array =
                ndarray::Array2::from_shape_vec(input_shape, attention_mask)
                    .map_err(|e| Error::Inference(e.to_string()))?;

            let input_ids_val = Value::from_array(input_ids_array)
                .map_err(|e| Error::Inference(e.to_string()))?;
            let attention_mask_val = Value::from_array(attention_mask_array)
                .map_err(|e| Error::Inference(e.to_string()))?;

            let mut session_guard = self.session.lock();
            let outputs = session_guard
                .run(ort::inputs![
                    "input_ids" => input_ids_val,
                    "attention_mask" => attention_mask_val
                ].map_err(|e| Error::Inference(e.to_string()))?)
                .map_err(|e| Error::Inference(e.to_string()))?;

            let logits_val = outputs
                .get("logits")
                .or_else(|| outputs.get("0"))
                .ok_or_else(|| Error::Inference("Ausgabe 'logits' nicht gefunden".into()))?;

            let tensor_view = logits_val
                .try_extract_tensor::<f32>()
                .map_err(|e| Error::Inference(e.to_string()))?;

            let mut scores = Vec::with_capacity(batch_size);
            for b in 0..batch_size {
                let raw_score = match tensor_view.shape().len() {
                    1 => tensor_view.get(b).copied().unwrap_or(0.0f32),
                    2 => tensor_view.get([b, 0]).copied().unwrap_or(0.0f32),
                    _ => 0.0f32,
                };
                // Sigmoid Normalisierung
                let sigmoid = 1.0f32 / (1.0f32 + (-raw_score).exp());
                scores.push(sigmoid);
            }

            Ok(scores)
        }
        #[cfg(not(feature = "onnx"))]
        {
            let _ = (query, docs);
            Err(Error::Unsupported(
                "ONNX-Feature ist nicht aktiviert in diesem Build".into(),
            ))
        }
    }
}

/// Universeller Cross-Encoder Wrapper, der entweder ONNX Runtime oder eine Fake/Passthrough-Engine nutzt.
pub enum CrossEncoderReranker {
    /// ONNX Runtime Engine.
    Onnx(OnnxReranker),
    /// Fake / Passthrough Engine für isolierten Betrieb ohne externe Abhängigkeiten.
    Fake(crate::fakes::FakeReranker),
}

impl CrossEncoderReranker {
    /// Erstellt einen Passthrough- / Fake-Reranker ohne externe Modell-Abhängigkeiten.
    #[must_use]
    pub fn passthrough() -> Self {
        Self::Fake(crate::fakes::FakeReranker::new())
    }

    /// Erstellt einen ONNX-gestützten Reranker.
    ///
    /// # Errors
    /// Gibt `Error` zurück, falls die Initialisierung scheitert.
    pub fn from_config(config: RerankerConfig) -> Result<Self> {
        Ok(Self::Onnx(OnnxReranker::new(config)?))
    }
}

impl Reranker for CrossEncoderReranker {
    fn score(&self, query: &str, docs: &[&str]) -> Result<Vec<f32>> {
        match self {
            Self::Onnx(r) => r.score(query, docs),
            Self::Fake(f) => f.score(query, docs),
        }
    }
}
