// FILE-CONTEXT
// STAND: 2026-09-15T16:10:40Z (SESSION: ec33599e)
// ZWECK: Real ONNX Runtime Cross-Encoder Inferenz-Backend.

#[cfg(feature = "onnx")]
use contextra_rank::PlattScaler;
#[cfg(feature = "onnx")]
use contextra_types::ContextraError;
#[cfg(feature = "onnx")]
use ort::value::Value;
#[cfg(feature = "onnx")]
use tokenizers::Tokenizer;
#[cfg(feature = "onnx")]
use tracing::warn;

#[cfg(feature = "onnx")]
use super::config::{PlattScaledSigmoid, RerankConfig, RerankResult};

// CONCURRENCY & LOCK HIERARCHY:
// `OnnxReranker` uses a single `parking_lot::Mutex<ort::session::Session>` lock.
// No nested locks exist anywhere within `contextra-embed`.
// Locks are acquired exclusively inside `spawn_blocking` calls for the duration of ONNX inference,
// preventing async executor starvation and eliminating deadlock risks.
//
// ARCHITECTURAL NOTE (SessionPool Separation):
// `OnnxReranker` uses an independent `Arc<Mutex<Session>>` session management scheme
// separate from `TextEmbedder`'s `Semaphore`-based pool. This intentional separation
// accounts for fundamental differences in model signatures and execution profiles:
// Cross-Encoder reranking operates on `(query, document)` sequence pairs requiring
// custom dynamic batching, whereas `TextEmbedder` executes single-text embeddings.
#[cfg(feature = "onnx")]
pub(super) struct OnnxReranker {
    config: RerankConfig,
    tokenizer: std::sync::Arc<Tokenizer>,
    // parking_lot::Mutex ist panic-safe (kein PoisonError), da es keinen
    // Poison-Mechanismus hat. Kein .unwrap()/.map_err() nötig. // unwrap
    session: std::sync::Arc<parking_lot::Mutex<ort::session::Session>>,
}

#[cfg(feature = "onnx")]
impl OnnxReranker {
    /// Erstellt eine neue `OnnxReranker`-Instanz.
    pub(super) fn new(config: RerankConfig) -> Result<Self, ContextraError> {
        if !config.model_path.exists() {
            return Err(ContextraError::InvalidInput(format!(
                "ONNX model file not found at {:?}",
                config.model_path
            )));
        }
        if !config.tokenizer_path.exists() {
            return Err(ContextraError::InvalidInput(format!(
                "Tokenizer file not found at {:?}",
                config.tokenizer_path
            )));
        }

        let tokenizer = Tokenizer::from_file(&config.tokenizer_path)
            .map_err(|e| ContextraError::Internal(format!("Failed to load tokenizer: {e}")))?;

        use ort::session::Session;
        let session = Session::builder()
            .map_err(|e| ContextraError::Internal(format!("ONNX session builder: {e}")))?
            .commit_from_file(&config.model_path)
            .map_err(|e| {
                ContextraError::Internal(format!(
                    "ONNX model load from {:?}: {e}",
                    config.model_path
                ))
            })?;

        Ok(Self {
            config,
            tokenizer: std::sync::Arc::new(tokenizer),
            session: std::sync::Arc::new(parking_lot::Mutex::new(session)),
        })
    }

    /// Rerankt Kandidaten für eine Abfrage.
    pub(super) async fn rerank(
        &self,
        query: &str,
        candidates: &[String],
        calibration: &PlattScaler,
    ) -> Result<Vec<RerankResult>, ContextraError> {
        if candidates.is_empty() {
            return Ok(vec![]);
        }

        let pairs: Vec<(String, String)> = candidates
            .iter()
            .map(|c| (query.to_string(), c.clone()))
            .collect();

        let session = std::sync::Arc::clone(&self.session);
        let tokenizer = std::sync::Arc::clone(&self.tokenizer);
        let max_length = self.config.max_length;
        let batch_size = self.config.batch_size;
        let calibration = calibration.clone();
        let scores = tokio::task::spawn_blocking(move || {
            Self::score_pairs_blocking(
                &session,
                &tokenizer,
                &pairs,
                max_length,
                batch_size,
                &calibration,
            )
        })
        .await
        .map_err(|e| ContextraError::Internal(format!("Rerank task panicked: {e:?}")))?
        .map_err(|e| ContextraError::Internal(format!("Rerank scoring failed: {e}")))?;

        let mut results: Vec<RerankResult> = scores
            .into_iter()
            .enumerate()
            .map(|(i, score)| RerankResult {
                original_index: i,
                score,
            })
            .collect();

        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        Ok(results)
    }

    fn score_pairs_blocking(
        session: &std::sync::Arc<parking_lot::Mutex<ort::session::Session>>,
        tokenizer: &Tokenizer,
        pairs: &[(String, String)],
        max_length: usize,
        batch_size: usize,
        calibration: &PlattScaledSigmoid,
    ) -> Result<Vec<f32>, String> {
        if pairs.is_empty() {
            return Ok(vec![]);
        }

        let batch_size = batch_size.max(1);
        let mut all_scores = Vec::with_capacity(pairs.len());

        for chunk in pairs.chunks(batch_size) {
            let chunk_scores =
                Self::score_batch(session, tokenizer, chunk, max_length, calibration)?;
            all_scores.extend(chunk_scores);
        }

        Ok(all_scores)
    }

    fn score_batch(
        session: &std::sync::Arc<parking_lot::Mutex<ort::session::Session>>,
        tokenizer: &Tokenizer,
        chunk: &[(String, String)],
        max_length: usize,
        calibration: &PlattScaledSigmoid,
    ) -> Result<Vec<f32>, String> {
        let mut encodings = Vec::with_capacity(chunk.len());

        for (query, candidate) in chunk {
            let encoding = tokenizer
                .encode((query.as_str(), candidate.as_str()), true)
                .map_err(|e| format!("Tokenization failed for pair: {e}"))?;

            let mut input_ids = encoding.get_ids().to_vec();
            let mut attention_mask = encoding.get_attention_mask().to_vec();
            let mut type_ids = encoding.get_type_ids().to_vec();

            if input_ids.len() > max_length {
                warn!(
                    tokens = input_ids.len(),
                    max = max_length,
                    "Reranker pair tokens exceed max_length and will be truncated."
                );
                input_ids.truncate(max_length);
                attention_mask.truncate(max_length);
                type_ids.truncate(max_length);
            }

            encodings.push((input_ids, attention_mask, type_ids));
        }

        let b_size = chunk.len();
        let max_seq_len = encodings
            .iter()
            .map(|(ids, _, _)| ids.len())
            .max()
            .unwrap_or(1);

        let mut input_ids_flat = vec![0i64; b_size * max_seq_len];
        let mut attention_mask_flat = vec![0i64; b_size * max_seq_len];
        let mut type_ids_flat = vec![0i64; b_size * max_seq_len];

        for (i, (ids, mask, tids)) in encodings.iter().enumerate() {
            for (j, &id) in ids.iter().enumerate() {
                input_ids_flat[i * max_seq_len + j] = id as i64;
            }
            for (j, &m) in mask.iter().enumerate() {
                attention_mask_flat[i * max_seq_len + j] = m as i64;
            }
            for (j, &t) in tids.iter().enumerate() {
                type_ids_flat[i * max_seq_len + j] = t as i64;
            }
        }

        let input_ids_tensor = Value::from_array(([b_size, max_seq_len], input_ids_flat))
            .map_err(|e| format!("Failed to create input_ids tensor: {e}"))?;
        let attention_mask_tensor = Value::from_array(([b_size, max_seq_len], attention_mask_flat))
            .map_err(|e| format!("Failed to create attention_mask tensor: {e}"))?;

        // SAFETY: parking_lot::Mutex ist nicht vergiftbar. Guard hält die
        // Mutex für die Dauer des ONNX-Inference-Calls. spawn_blocking
        // garantiert dass dies einen blocking thread nutzt (kein async starvation).
        let mut guard = session.lock();

        let has_token_type_ids = guard
            .inputs()
            .iter()
            .any(|input| input.name() == "token_type_ids");

        let result = {
            let outputs = if has_token_type_ids {
                let type_ids_tensor = Value::from_array(([b_size, max_seq_len], type_ids_flat))
                    .map_err(|e| format!("Failed to create token_type_ids tensor: {e}"))?;
                guard
                    .run(ort::inputs![
                        "input_ids" => input_ids_tensor,
                        "attention_mask" => attention_mask_tensor,
                        "token_type_ids" => type_ids_tensor,
                    ])
                    .map_err(|e| format!("ONNX reranker inference failed: {e}"))?
            } else {
                guard
                    .run(ort::inputs![
                        "input_ids" => input_ids_tensor,
                        "attention_mask" => attention_mask_tensor,
                    ])
                    .map_err(|e| format!("ONNX reranker inference failed: {e}"))?
            };

            if let Some(out) = outputs.get("logits") {
                let (shape, data) = out
                    .try_extract_tensor::<f32>()
                    .map_err(|e| format!("Failed to extract output tensor: {e}"))?;
                Self::extract_scores_from_tensor_calibrated(shape, data, b_size, calibration)
            } else if let Some((_, out)) = outputs.iter().next() {
                let (shape, data) = out
                    .try_extract_tensor::<f32>()
                    .map_err(|e| format!("Failed to extract output tensor: {e}"))?;
                Self::extract_scores_from_tensor_calibrated(shape, data, b_size, calibration)
            } else {
                Err("Reranker model produced no outputs".into())
            }
        };

        result
    }

    #[allow(dead_code)]
    pub(super) fn extract_scores_from_tensor(
        shape: &[i64],
        data: &[f32],
        b_size: usize,
    ) -> Result<Vec<f32>, String> {
        if shape.is_empty() || shape[0] as usize != b_size {
            return Err(format!(
                "Output tensor batch size mismatch: expected {b_size}, shape {:?}",
                shape
            ));
        }

        Self::extract_scores_from_tensor_calibrated(
            shape,
            data,
            b_size,
            &PlattScaledSigmoid::identity(),
        )
    }

    fn extract_scores_from_tensor_calibrated(
        shape: &[i64],
        data: &[f32],
        b_size: usize,
        calibration: &PlattScaledSigmoid,
    ) -> Result<Vec<f32>, String> {
        if shape.is_empty() || shape[0] as usize != b_size {
            return Err(format!(
                "Output tensor batch size mismatch: expected {b_size}, shape {:?}",
                shape
            ));
        }

        // AI-TAG[CALIBRATION][MINOR] AGT-EMBED-62093e61 [RESOLVED] (TS: 2026-09-06T12:00:00Z)
        // Kalibrierungslücke: Platt-Scaling nun via record_outcome() + fitted_calibration.
        let transform = |x: f32| -> f32 { calibration.transform(x) };
        let mut scores = Vec::with_capacity(b_size);

        match shape.len() {
            1 => {
                if data.len() != b_size {
                    return Err(format!(
                        "Output tensor len {} != batch size {b_size}",
                        data.len()
                    ));
                }
                for &logit in data {
                    scores.push(transform(logit));
                }
            }
            2 => {
                let cols = shape[1] as usize;
                if data.len() != b_size * cols {
                    return Err(format!(
                        "Output tensor len {} != expected {}",
                        data.len(),
                        b_size * cols
                    ));
                }
                if cols == 1 {
                    for &logit in data {
                        scores.push(transform(logit));
                    }
                } else if cols == 2 {
                    for i in 0..b_size {
                        let l0 = data[i * 2];
                        let l1 = data[i * 2 + 1];
                        scores.push(transform(l1 - l0));
                    }
                } else {
                    for i in 0..b_size {
                        scores.push(transform(data[i * cols]));
                    }
                }
            }
            _ => {
                return Err(format!("Unsupported output shape: {:?}", shape));
            }
        }

        Ok(scores)
    }
}
