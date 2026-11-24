//! ONNX Embedding Provider für In-Process Inferenz.
//!
//! Lädt Transformer-Modelle im ONNX-Format und erzeugt normalisierte Vektoren
//! mit konfigurierbarem Pooling (Mean / CLS).

use crate::config::EmbedderConfig;
#[cfg(feature = "onnx")]
use crate::config::PoolingMode;
use crate::error::{Error, Result};
use crate::traits::Embedder;

#[cfg(feature = "onnx")]
use ort::{session::Session, value::Value};
#[cfg(feature = "onnx")]
use parking_lot::Mutex;
#[cfg(feature = "onnx")]
use std::sync::Arc;
#[cfg(feature = "onnx")]
use tokenizers::Tokenizer;

/// ONNX-basierter Embedding-Provider mit Session-Pool und Tokenizer.
pub struct OnnxEmbedder {
    config: EmbedderConfig,
    #[cfg(feature = "onnx")]
    tokenizer: Tokenizer,
    #[cfg(feature = "onnx")]
    session: Arc<Mutex<Session>>,
}

impl OnnxEmbedder {
    /// Initialisiert einen neuen `OnnxEmbedder` anhand der übergebenen Konfiguration.
    ///
    /// # Errors
    /// Gibt `Error` zurück, wenn Validierung, Hash-Verifikation, Tokenizer oder ONNX-Session fehlschlagen.
    pub fn new(config: EmbedderConfig) -> Result<Self> {
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
    pub fn config(&self) -> &EmbedderConfig {
        &self.config
    }
}

impl Embedder for OnnxEmbedder {
    fn embed(&self, text: &str) -> Result<Vec<f32>> {
        #[cfg(feature = "onnx")]
        {
            let batch = self.embed_batch(&[text])?;
            match batch.into_iter().next() {
                Some(vec) => Ok(vec),
                None => Err(Error::Inference("Leere Rückgabe beim Embedding".into())),
            }
        }
        #[cfg(not(feature = "onnx"))]
        {
            let _ = text;
            Err(Error::Unsupported(
                "ONNX-Feature ist nicht aktiviert in diesem Build".into(),
            ))
        }
    }

    fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        #[cfg(feature = "onnx")]
        {
            if texts.is_empty() {
                return Ok(Vec::new());
            }

            let mut encoding_inputs = Vec::with_capacity(texts.len());
            for &t in texts {
                encoding_inputs.push(t);
            }

            let encodings = self
                .tokenizer
                .encode_batch(encoding_inputs, true)
                .map_err(|e| Error::Tokenization(e.to_string()))?;

            let batch_size = encodings.len();
            let seq_len = encodings
                .first()
                .map_or(0, |enc| enc.get_ids().len())
                .min(self.config.max_sequence_length);

            if seq_len == 0 {
                return Ok(vec![vec![0.0f32; self.config.expected_dimension]; batch_size]);
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

            let last_hidden_state = outputs
                .get("last_hidden_state")
                .or_else(|| outputs.get("0"))
                .ok_or_else(|| Error::Inference("Ausgabe 'last_hidden_state' nicht gefunden".into()))?;

            let tensor_view = last_hidden_state
                .try_extract_tensor::<f32>()
                .map_err(|e| Error::Inference(e.to_string()))?;

            let shape = tensor_view.shape();
            let hidden_dim = match shape.get(2) {
                Some(&dim) => dim,
                None => {
                    return Err(Error::Inference(
                        "Ungültiger Shape für last_hidden_state".into(),
                    ))
                }
            };

            if hidden_dim != self.config.expected_dimension {
                return Err(Error::InvalidDimension {
                    expected: self.config.expected_dimension,
                    actual: hidden_dim,
                });
            }

            let mut batch_embeddings = Vec::with_capacity(batch_size);

            for b in 0..batch_size {
                let mask = match encodings.get(b) {
                    Some(enc) => enc.get_attention_mask(),
                    None => &[],
                };

                let mut pooled = vec![0.0f32; hidden_dim];
                match self.config.pooling {
                    PoolingMode::Cls => {
                        for h in 0..hidden_dim {
                            if let Some(&val) = tensor_view.get([b, 0, h]) {
                                if let Some(target) = pooled.get_mut(h) {
                                    *target = val;
                                }
                            }
                        }
                    }
                    PoolingMode::Mean => {
                        let mut valid_tokens = 0usize;
                        for s in 0..seq_len {
                            let is_valid = mask.get(s).copied().unwrap_or(0) == 1;
                            if is_valid {
                                valid_tokens = valid_tokens.saturating_add(1);
                                for h in 0..hidden_dim {
                                    if let Some(&val) = tensor_view.get([b, s, h]) {
                                        if let Some(target) = pooled.get_mut(h) {
                                            *target += val;
                                        }
                                    }
                                }
                            }
                        }
                        if valid_tokens > 0 {
                            let factor = 1.0f32 / (valid_tokens as f32);
                            for val in &mut pooled {
                                *val *= factor;
                            }
                        }
                    }
                }

                if self.config.normalize_l2 {
                    let norm_sq: f32 = pooled.iter().map(|&x| x * x).sum();
                    let norm = norm_sq.sqrt();
                    if norm > 1e-6 {
                        let inv_norm = 1.0f32 / norm;
                        for val in &mut pooled {
                            *val *= inv_norm;
                        }
                    }
                }

                batch_embeddings.push(pooled);
            }

            Ok(batch_embeddings)
        }
        #[cfg(not(feature = "onnx"))]
        {
            let _ = texts;
            Err(Error::Unsupported(
                "ONNX-Feature ist nicht aktiviert in diesem Build".into(),
            ))
        }
    }

    fn dimension(&self) -> usize {
        self.config.expected_dimension
    }
}
