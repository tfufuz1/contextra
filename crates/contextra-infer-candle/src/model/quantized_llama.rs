// FILE-CONTEXT
// STAND: 2026-09-22T00:00:00Z
// ZWECK: Forked Candle Quantized Llama model with explicit KvState management (Spec §9.2 Stufe B).
// INVARIANTEN: Zero-Panic doctrine (no .unwrap() or .expect() in production code);
// Full access to layer-wise KV cache tensors via export_kv_state, import_kv_state, and truncate_kv_cache.

use crate::kv_state::{KvState, LayerKv};
use candle_core::quantized::{gguf_file, QMatMul, QTensor};
use candle_core::{DType, Device, IndexOp, Tensor};
use candle_nn::{Embedding, Module};
use candle_transformers::quantized_nn::RmsNorm;
use contextra_core::ContextraError;
use std::collections::HashMap;

pub const MAX_SEQ_LEN: usize = 4096;

fn repeat_kv(x: Tensor, n_rep: usize) -> Result<Tensor, ContextraError> {
    if n_rep == 1 {
        Ok(x)
    } else {
        let (b_sz, n_kv_head, seq_len, head_dim) = x
            .dims4()
            .map_err(|e| ContextraError::Internal(format!("Failed to get dims4: {e}")))?;
        let x = x
            .unsqueeze(2)
            .map_err(|e| ContextraError::Internal(format!("Failed to unsqueeze tensor: {e}")))?
            .expand((b_sz, n_kv_head, n_rep, seq_len, head_dim))
            .map_err(|e| ContextraError::Internal(format!("Failed to expand tensor: {e}")))?
            .reshape((b_sz, n_kv_head * n_rep, seq_len, head_dim))
            .map_err(|e| ContextraError::Internal(format!("Failed to reshape tensor: {e}")))?;
        Ok(x)
    }
}

fn precomput_freqs_cis(
    head_dim: usize,
    freq_base: f32,
    device: &Device,
) -> Result<(Tensor, Tensor), ContextraError> {
    let theta: Vec<_> = (0..head_dim)
        .step_by(2)
        .map(|i| 1f32 / freq_base.powf(i as f32 / head_dim as f32))
        .collect();
    let theta = Tensor::new(theta.as_slice(), device)
        .map_err(|e| ContextraError::Internal(format!("theta tensor creation error: {e}")))?;
    let idx_theta = Tensor::arange(0u32, MAX_SEQ_LEN as u32, device)
        .map_err(|e| ContextraError::Internal(format!("arange error: {e}")))?
        .to_dtype(DType::F32)
        .map_err(|e| ContextraError::Internal(format!("to_dtype error: {e}")))?
        .reshape((MAX_SEQ_LEN, 1))
        .map_err(|e| ContextraError::Internal(format!("reshape error: {e}")))?
        .matmul(
            &theta
                .reshape((1, theta.elem_count()))
                .map_err(|e| ContextraError::Internal(format!("theta reshape error: {e}")))?,
        )
        .map_err(|e| ContextraError::Internal(format!("matmul error: {e}")))?;
    let cos = idx_theta
        .cos()
        .map_err(|e| ContextraError::Internal(format!("cos error: {e}")))?;
    let sin = idx_theta
        .sin()
        .map_err(|e| ContextraError::Internal(format!("sin error: {e}")))?;
    Ok((cos, sin))
}

fn build_causal_mask(
    seq_len: usize,
    index_pos: usize,
    device: &Device,
) -> Result<Tensor, ContextraError> {
    let mask: Vec<_> = (0..seq_len)
        .flat_map(|i| {
            (0..seq_len + index_pos).map(move |j| if j <= i + index_pos { 0u8 } else { 1u8 })
        })
        .collect();
    Tensor::from_slice(&mask, (seq_len, seq_len + index_pos), device)
        .map_err(|e| ContextraError::Internal(format!("Failed to create causal mask tensor: {e}")))
}

#[derive(Debug, Clone)]
pub struct QuantizedMatMul {
    inner: QMatMul,
    span: tracing::Span,
}

impl QuantizedMatMul {
    fn from_qtensor(qtensor: QTensor) -> Result<Self, ContextraError> {
        let inner = QMatMul::from_qtensor(qtensor)
            .map_err(|e| ContextraError::Internal(format!("Failed to build QMatMul: {e}")))?;
        let span = tracing::span!(tracing::Level::TRACE, "qmatmul");
        Ok(Self { inner, span })
    }

    fn forward(&self, xs: &Tensor) -> Result<Tensor, ContextraError> {
        let _enter = self.span.enter();
        self.inner
            .forward(xs)
            .map_err(|e| ContextraError::Internal(format!("QMatMul forward error: {e}")))
    }
}

#[derive(Debug, Clone)]
struct Mlp {
    feed_forward_w1: QuantizedMatMul,
    feed_forward_w2: QuantizedMatMul,
    feed_forward_w3: QuantizedMatMul,
}

impl Mlp {
    fn forward(&self, xs: &Tensor) -> Result<Tensor, ContextraError> {
        let w1 = self.feed_forward_w1.forward(xs)?;
        let w3 = self.feed_forward_w3.forward(xs)?;
        let silu_w1 = candle_nn::ops::silu(&w1)
            .map_err(|e| ContextraError::Internal(format!("SiLU error: {e}")))?;
        let mul = (silu_w1 * w3)
            .map_err(|e| ContextraError::Internal(format!("Tensor multiplication error: {e}")))?;
        self.feed_forward_w2.forward(&mul)
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
enum MlpOrMoe {
    Mlp(Mlp),
    MoE {
        n_expert_used: usize,
        feed_forward_gate_inp: QuantizedMatMul,
        experts: Vec<Mlp>,
    },
}

impl MlpOrMoe {
    fn forward(&self, xs: &Tensor) -> Result<Tensor, ContextraError> {
        match self {
            Self::MoE {
                feed_forward_gate_inp,
                experts,
                n_expert_used,
            } => {
                let (b_size, seq_len, hidden_dim) = xs
                    .dims3()
                    .map_err(|e| ContextraError::Internal(format!("Failed dims3: {e}")))?;
                let xs_reshaped = xs
                    .reshape(((), hidden_dim))
                    .map_err(|e| ContextraError::Internal(format!("Failed reshape: {e}")))?;
                let router_logits = feed_forward_gate_inp.forward(&xs_reshaped)?;
                let routing_weights = candle_nn::ops::softmax_last_dim(&router_logits)
                    .map_err(|e| ContextraError::Internal(format!("Softmax error: {e}")))?;

                let routing_weights_vec = routing_weights
                    .to_dtype(DType::F32)
                    .map_err(|e| ContextraError::Internal(format!("to_dtype error: {e}")))?
                    .to_vec2::<f32>()
                    .map_err(|e| ContextraError::Internal(format!("to_vec2 error: {e}")))?;

                let mut top_x = vec![vec![]; experts.len()];
                let mut selected_rws = vec![vec![]; experts.len()];
                for (row_idx, rw) in routing_weights_vec.iter().enumerate() {
                    let mut dst = (0..rw.len() as u32).collect::<Vec<u32>>();
                    dst.sort_by(|&i, &j| rw[j as usize].total_cmp(&rw[i as usize]));
                    let mut sum_routing_weights = 0f32;
                    for &expert_idx in dst.iter().take(*n_expert_used) {
                        let expert_idx = expert_idx as usize;
                        let routing_weight = rw[expert_idx];
                        sum_routing_weights += routing_weight;
                        top_x[expert_idx].push(row_idx as u32);
                    }
                    for &expert_idx in dst.iter().take(*n_expert_used) {
                        let expert_idx = expert_idx as usize;
                        let routing_weight = rw[expert_idx];
                        let rw_norm = if sum_routing_weights > 0.0 {
                            routing_weight / sum_routing_weights
                        } else {
                            0.0
                        };
                        selected_rws[expert_idx].push(rw_norm);
                    }
                }

                let mut ys = xs_reshaped
                    .zeros_like()
                    .map_err(|e| ContextraError::Internal(format!("zeros_like error: {e}")))?;
                for (expert_idx, expert_layer) in experts.iter().enumerate() {
                    let top_x_indices = &top_x[expert_idx];
                    if top_x_indices.is_empty() {
                        continue;
                    }
                    let top_x_tensor = Tensor::new(top_x_indices.as_slice(), xs.device())
                        .map_err(|e| ContextraError::Internal(format!("Tensor::new error: {e}")))?;
                    let selected_rws_tensor =
                        Tensor::new(selected_rws[expert_idx].as_slice(), xs.device())
                            .map_err(|e| ContextraError::Internal(format!("Tensor::new error: {e}")))?
                            .reshape(((), 1))
                            .map_err(|e| ContextraError::Internal(format!("reshape error: {e}")))?;

                    let current_state = xs_reshaped
                        .index_select(&top_x_tensor, 0)
                        .map_err(|e| ContextraError::Internal(format!("index_select error: {e}")))?
                        .reshape(((), hidden_dim))
                        .map_err(|e| ContextraError::Internal(format!("reshape error: {e}")))?;
                    let current_hidden_states = expert_layer.forward(&current_state)?;
                    let current_hidden_states = current_hidden_states
                        .broadcast_mul(&selected_rws_tensor)
                        .map_err(|e| ContextraError::Internal(format!("broadcast_mul error: {e}")))?;
                    ys = ys
                        .index_add(&top_x_tensor, &current_hidden_states, 0)
                        .map_err(|e| ContextraError::Internal(format!("index_add error: {e}")))?;
                }

                let ys = ys
                    .reshape((b_size, seq_len, hidden_dim))
                    .map_err(|e| ContextraError::Internal(format!("reshape error: {e}")))?;
                Ok(ys)
            }
            Self::Mlp(mlp) => mlp.forward(xs),
        }
    }
}

/// Transformer layer weights wrapping QMatMul attention projections, RMS norms, and KV cache.
#[derive(Debug, Clone)]
pub struct LayerWeights {
    pub attention_wq: QuantizedMatMul,
    pub attention_wk: QuantizedMatMul,
    pub attention_wv: QuantizedMatMul,
    pub attention_wo: QuantizedMatMul,
    pub attention_norm: RmsNorm,
    mlp_or_moe: MlpOrMoe,
    pub ffn_norm: RmsNorm,
    pub n_head: usize,
    pub n_kv_head: usize,
    pub head_dim: usize,
    pub rope_is_neox: bool,
    pub cos: Tensor,
    pub sin: Tensor,
    pub neg_inf: Tensor,
    pub kv_cache: Option<(Tensor, Tensor)>,
    span_attn: tracing::Span,
    span_rot: tracing::Span,
    span_mlp: tracing::Span,
}

impl LayerWeights {
    fn apply_rotary_emb(&self, x: &Tensor, index_pos: usize) -> Result<Tensor, ContextraError> {
        let _enter = self.span_rot.enter();
        let (_b_sz, _n_head, seq_len, _n_embd) = x
            .dims4()
            .map_err(|e| ContextraError::Internal(format!("Failed dims4 in RoPE: {e}")))?;
        let cos = self
            .cos
            .narrow(0, index_pos, seq_len)
            .map_err(|e| ContextraError::Internal(format!("Narrow cos error: {e}")))?;
        let sin = self
            .sin
            .narrow(0, index_pos, seq_len)
            .map_err(|e| ContextraError::Internal(format!("Narrow sin error: {e}")))?;
        let x_cont = x
            .contiguous()
            .map_err(|e| ContextraError::Internal(format!("Contiguous error: {e}")))?;
        if self.rope_is_neox {
            candle_nn::rotary_emb::rope(&x_cont, &cos, &sin)
                .map_err(|e| ContextraError::Internal(format!("RoPE error: {e}")))
        } else {
            candle_nn::rotary_emb::rope_i(&x_cont, &cos, &sin)
                .map_err(|e| ContextraError::Internal(format!("RoPE_i error: {e}")))
        }
    }

    fn forward_attn(
        &mut self,
        x: &Tensor,
        mask: Option<&Tensor>,
        index_pos: usize,
    ) -> Result<Tensor, ContextraError> {
        let _enter = self.span_attn.enter();
        let (b_sz, seq_len, _n_embd) = x
            .dims3()
            .map_err(|e| ContextraError::Internal(format!("dims3 error: {e}")))?;
        let q = self.attention_wq.forward(x)?;
        let k = self.attention_wk.forward(x)?;
        let v = self.attention_wv.forward(x)?;

        let q = q
            .reshape((b_sz, seq_len, self.n_head, self.head_dim))
            .map_err(|e| ContextraError::Internal(format!("q reshape error: {e}")))?
            .transpose(1, 2)
            .map_err(|e| ContextraError::Internal(format!("q transpose error: {e}")))?;
        let k = k
            .reshape((b_sz, seq_len, self.n_kv_head, self.head_dim))
            .map_err(|e| ContextraError::Internal(format!("k reshape error: {e}")))?
            .transpose(1, 2)
            .map_err(|e| ContextraError::Internal(format!("k transpose error: {e}")))?;
        let v = v
            .reshape((b_sz, seq_len, self.n_kv_head, self.head_dim))
            .map_err(|e| ContextraError::Internal(format!("v reshape error: {e}")))?
            .transpose(1, 2)
            .map_err(|e| ContextraError::Internal(format!("v transpose error: {e}")))?
            .contiguous()
            .map_err(|e| ContextraError::Internal(format!("v contiguous error: {e}")))?;

        let q = self.apply_rotary_emb(&q, index_pos)?;
        let k = self.apply_rotary_emb(&k, index_pos)?;

        let (k, v) = match &self.kv_cache {
            None => (k, v),
            Some((k_cache, v_cache)) => {
                if index_pos == 0 {
                    (k, v)
                } else {
                    let k_cat = Tensor::cat(&[k_cache, &k], 2)
                        .map_err(|e| ContextraError::Internal(format!("k cat error: {e}")))?;
                    let v_cat = Tensor::cat(&[v_cache, &v], 2)
                        .map_err(|e| ContextraError::Internal(format!("v cat error: {e}")))?;
                    (k_cat, v_cat)
                }
            }
        };
        self.kv_cache = Some((k.clone(), v.clone()));

        let k_rep = repeat_kv(k, self.n_head / self.n_kv_head)?;
        let v_rep = repeat_kv(v, self.n_head / self.n_kv_head)?;

        let kt = k_rep
            .t()
            .map_err(|e| ContextraError::Internal(format!("k transpose error: {e}")))?;
        let att = (q
            .matmul(&kt)
            .map_err(|e| ContextraError::Internal(format!("q @ k error: {e}")))?
            / (self.head_dim as f64).sqrt())
        .map_err(|e| ContextraError::Internal(format!("scale att error: {e}")))?;

        let att = match mask {
            None => att,
            Some(m) => {
                let mask_broadcast = m
                    .broadcast_as(att.shape())
                    .map_err(|e| ContextraError::Internal(format!("mask broadcast error: {e}")))?;
                att.where_cond(
                    &self.neg_inf.broadcast_as(att.shape()).map_err(|e| {
                        ContextraError::Internal(format!("neg_inf broadcast error: {e}"))
                    })?,
                    &mask_broadcast,
                )
                .map_err(|e| ContextraError::Internal(format!("mask where_cond error: {e}")))?
            }
        };

        let att = candle_nn::ops::softmax_last_dim(&att)
            .map_err(|e| ContextraError::Internal(format!("softmax att error: {e}")))?;
        let y = att
            .matmul(&v_rep)
            .map_err(|e| ContextraError::Internal(format!("att @ v error: {e}")))?;
        let y = y
            .transpose(1, 2)
            .map_err(|e| ContextraError::Internal(format!("y transpose error: {e}")))?
            .reshape((b_sz, seq_len, self.n_head * self.head_dim))
            .map_err(|e| ContextraError::Internal(format!("y reshape error: {e}")))?;

        self.attention_wo.forward(&y)
    }
}

/// Quantized Llama model weights and layer structures with explicit KvState management.
#[derive(Debug, Clone)]
pub struct ModelWeights {
    pub tok_embeddings: Embedding,
    pub layers: Vec<LayerWeights>,
    pub norm: RmsNorm,
    pub output: QuantizedMatMul,
    pub masks: HashMap<(usize, usize), Tensor>,
    span: tracing::Span,
    span_output: tracing::Span,
}

impl ModelWeights {
    pub fn from_gguf<R: std::io::Seek + std::io::Read>(
        ct: gguf_file::Content,
        reader: &mut R,
        device: &Device,
    ) -> Result<Self, ContextraError> {
        let md_get = |key: &str| -> Result<&gguf_file::Value, ContextraError> {
            ct.metadata.get(key).ok_or_else(|| {
                ContextraError::InvalidInput(format!("Missing GGUF metadata key: {key}"))
            })
        };

        let head_count = md_get("llama.attention.head_count").and_then(|v| {
            v.to_u32()
                .map_err(|e| ContextraError::InvalidInput(e.to_string()))
        })? as usize;
        let head_count_kv = md_get("llama.attention.head_count_kv").and_then(|v| {
            v.to_u32()
                .map_err(|e| ContextraError::InvalidInput(e.to_string()))
        })? as usize;
        let block_count = md_get("llama.block_count").and_then(|v| {
            v.to_u32()
                .map_err(|e| ContextraError::InvalidInput(e.to_string()))
        })? as usize;
        let embedding_length = md_get("llama.embedding_length").and_then(|v| {
            v.to_u32()
                .map_err(|e| ContextraError::InvalidInput(e.to_string()))
        })? as usize;
        let rms_norm_eps = md_get("llama.attention.layer_norm_rms_epsilon").and_then(|v| {
            v.to_f32()
                .map_err(|e| ContextraError::InvalidInput(e.to_string()))
        })?;

        let tok_embeddings = ct
            .tensor(reader, "token_embd.weight", device)
            .map_err(|e| ContextraError::Internal(format!("Failed token_embd.weight: {e}")))?;
        let tok_embeddings = tok_embeddings
            .dequantize(device)
            .map_err(|e| ContextraError::Internal(format!("Failed dequantize embeddings: {e}")))?;
        let norm = ct
            .tensor(reader, "output_norm.weight", device)
            .map_err(|e| ContextraError::Internal(format!("Failed output_norm.weight: {e}")))?;
        let output = ct
            .tensor(reader, "output.weight", device)
            .map_err(|e| ContextraError::Internal(format!("Failed output.weight: {e}")))?;

        let mut layers = Vec::with_capacity(block_count);
        let head_dim = embedding_length / head_count;

        let neg_inf = Tensor::new(f32::NEG_INFINITY, device)
            .map_err(|e| ContextraError::Internal(format!("neg_inf tensor creation failed: {e}")))?;

        let freqs_cis = precomput_freqs_cis(head_dim, 10000.0, device)?;

        for layer_idx in 0..block_count {
            let prefix = format!("blk.{layer_idx}");
            let attention_wq = ct
                .tensor(reader, &format!("{prefix}.attn_q.weight"), device)
                .map_err(|e| ContextraError::Internal(e.to_string()))?;
            let attention_wk = ct
                .tensor(reader, &format!("{prefix}.attn_k.weight"), device)
                .map_err(|e| ContextraError::Internal(e.to_string()))?;
            let attention_wv = ct
                .tensor(reader, &format!("{prefix}.attn_v.weight"), device)
                .map_err(|e| ContextraError::Internal(e.to_string()))?;
            let attention_wo = ct
                .tensor(reader, &format!("{prefix}.attn_output.weight"), device)
                .map_err(|e| ContextraError::Internal(e.to_string()))?;

            let feed_forward_w1 = ct
                .tensor(reader, &format!("{prefix}.ffn_gate.weight"), device)
                .map_err(|e| ContextraError::Internal(e.to_string()))?;
            let feed_forward_w2 = ct
                .tensor(reader, &format!("{prefix}.ffn_down.weight"), device)
                .map_err(|e| ContextraError::Internal(e.to_string()))?;
            let feed_forward_w3 = ct
                .tensor(reader, &format!("{prefix}.ffn_up.weight"), device)
                .map_err(|e| ContextraError::Internal(e.to_string()))?;

            let attention_norm = ct
                .tensor(reader, &format!("{prefix}.attn_norm.weight"), device)
                .map_err(|e| ContextraError::Internal(e.to_string()))?;
            let ffn_norm = ct
                .tensor(reader, &format!("{prefix}.ffn_norm.weight"), device)
                .map_err(|e| ContextraError::Internal(e.to_string()))?;

            let span_attn = tracing::span!(tracing::Level::TRACE, "attn");
            let span_rot = tracing::span!(tracing::Level::TRACE, "attn-rot");
            let span_mlp = tracing::span!(tracing::Level::TRACE, "attn-mlp");

            layers.push(LayerWeights {
                attention_wq: QuantizedMatMul::from_qtensor(attention_wq)?,
                attention_wk: QuantizedMatMul::from_qtensor(attention_wk)?,
                attention_wv: QuantizedMatMul::from_qtensor(attention_wv)?,
                attention_wo: QuantizedMatMul::from_qtensor(attention_wo)?,
                attention_norm: RmsNorm::from_qtensor(attention_norm, rms_norm_eps as f64)
                    .map_err(|e| ContextraError::Internal(e.to_string()))?,
                mlp_or_moe: MlpOrMoe::Mlp(Mlp {
                    feed_forward_w1: QuantizedMatMul::from_qtensor(feed_forward_w1)?,
                    feed_forward_w2: QuantizedMatMul::from_qtensor(feed_forward_w2)?,
                    feed_forward_w3: QuantizedMatMul::from_qtensor(feed_forward_w3)?,
                }),
                ffn_norm: RmsNorm::from_qtensor(ffn_norm, rms_norm_eps as f64)
                    .map_err(|e| ContextraError::Internal(e.to_string()))?,
                n_head: head_count,
                n_kv_head: head_count_kv,
                head_dim,
                rope_is_neox: true,
                cos: freqs_cis.0.clone(),
                sin: freqs_cis.1.clone(),
                neg_inf: neg_inf.clone(),
                kv_cache: None,
                span_attn,
                span_rot,
                span_mlp,
            });
        }

        let span = tracing::span!(tracing::Level::TRACE, "model");
        let span_output = tracing::span!(tracing::Level::TRACE, "output");

        Ok(Self {
            tok_embeddings: Embedding::new(tok_embeddings, embedding_length),
            layers,
            norm: RmsNorm::from_qtensor(norm, rms_norm_eps as f64)
                .map_err(|e| ContextraError::Internal(e.to_string()))?,
            output: QuantizedMatMul::from_qtensor(output)?,
            masks: HashMap::new(),
            span,
            span_output,
        })
    }

    fn mask(
        &mut self,
        seq_len: usize,
        index_pos: usize,
        device: &Device,
    ) -> Result<Tensor, ContextraError> {
        let kv_len = index_pos + seq_len;
        if let Some(mask) = self.masks.get(&(seq_len, kv_len)) {
            Ok(mask.clone())
        } else {
            let mask = build_causal_mask(seq_len, index_pos, device)?;
            self.masks.insert((seq_len, kv_len), mask.clone());
            Ok(mask)
        }
    }

    /// Exports the current KV cache across all model layers into a `KvState`.
    pub fn export_kv_state(&self) -> Result<KvState, ContextraError> {
        let mut layer_kvs = Vec::with_capacity(self.layers.len());
        let mut max_pos = 0;

        for layer in &self.layers {
            if let Some((k, v)) = &layer.kv_cache {
                if k.dims().len() >= 3 {
                    max_pos = max_pos.max(k.dims()[2]);
                }
                layer_kvs.push(LayerKv::new(k.clone(), v.clone()));
            } else {
                return Err(ContextraError::InvalidInput(
                    "Cannot export KvState when layer kv_cache is uninitialized".to_string(),
                ));
            }
        }

        Ok(KvState::new(layer_kvs, max_pos))
    }

    /// Imports a `KvState` into all model layers.
    pub fn import_kv_state(&mut self, state: &KvState) -> Result<(), ContextraError> {
        if self.layers.len() != state.layers.len() {
            return Err(ContextraError::InvalidInput(format!(
                "Layer count mismatch: model has {} layers, imported state has {}",
                self.layers.len(),
                state.layers.len()
            )));
        }

        for (layer, layer_kv) in self.layers.iter_mut().zip(&state.layers) {
            layer.kv_cache = Some((layer_kv.k.clone(), layer_kv.v.clone()));
        }

        Ok(())
    }

    /// Truncates the cached KV tensors across all layers to `pos` tokens.
    pub fn truncate_kv_cache(&mut self, pos: usize) -> Result<(), ContextraError> {
        for layer in self.layers.iter_mut() {
            if let Some((k, v)) = &mut layer.kv_cache {
                if pos == 0 {
                    layer.kv_cache = None;
                } else {
                    let k_len = k.dims().get(2).copied().unwrap_or(0);
                    if pos < k_len {
                        let new_k = k.narrow(2, 0, pos).map_err(|e| {
                            ContextraError::Internal(format!("Failed to truncate key tensor: {e}"))
                        })?;
                        let new_v = v.narrow(2, 0, pos).map_err(|e| {
                            ContextraError::Internal(format!("Failed to truncate value tensor: {e}"))
                        })?;
                        layer.kv_cache = Some((new_k, new_v));
                    }
                }
            }
        }
        Ok(())
    }

    /// Clears the KV cache across all layers.
    pub fn clear_kv_cache(&mut self) {
        for layer in self.layers.iter_mut() {
            layer.kv_cache = None;
        }
    }

    /// Performs forward pass through token embeddings, attention layers, norms, and output projection.
    pub fn forward(&mut self, x: &Tensor, index_pos: usize) -> Result<Tensor, ContextraError> {
        let (_b_sz, seq_len) = x
            .dims2()
            .map_err(|e| ContextraError::Internal(format!("dims2 error: {e}")))?;
        let mask = if seq_len == 1 {
            None
        } else {
            Some(self.mask(seq_len, index_pos, x.device())?)
        };
        let _enter = self.span.enter();
        let mut layer_in = self
            .tok_embeddings
            .forward(x)
            .map_err(|e| ContextraError::Internal(format!("tok_embeddings error: {e}")))?;

        for layer in self.layers.iter_mut() {
            let x = layer_in;
            let residual = &x;
            let x_norm = layer
                .attention_norm
                .forward(&x)
                .map_err(|e| ContextraError::Internal(format!("attn_norm error: {e}")))?;
            let attn = layer.forward_attn(&x_norm, mask.as_ref(), index_pos)?;
            let x_res = (attn + residual)
                .map_err(|e| ContextraError::Internal(format!("residual add error: {e}")))?;

            let _enter = layer.span_mlp.enter();
            let residual = &x_res;
            let x_ffn_norm = layer
                .ffn_norm
                .forward(&x_res)
                .map_err(|e| ContextraError::Internal(format!("ffn_norm error: {e}")))?;
            let x_mlp = layer.mlp_or_moe.forward(&x_ffn_norm)?;
            let x_out = (x_mlp + residual)
                .map_err(|e| ContextraError::Internal(format!("mlp residual add error: {e}")))?;
            layer_in = x_out;
        }

        let x_norm = self
            .norm
            .forward(&layer_in)
            .map_err(|e| ContextraError::Internal(format!("output norm error: {e}")))?;
        let x_last = x_norm
            .i((.., seq_len - 1, ..))
            .map_err(|e| ContextraError::Internal(format!("slice last token error: {e}")))?;
        let _enter = self.span_output.enter();
        self.output.forward(&x_last)
    }
}
