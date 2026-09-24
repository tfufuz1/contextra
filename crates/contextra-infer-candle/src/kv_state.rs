// FILE-CONTEXT
// STAND: 2026-09-22T00:00:00Z
// ZWECK: KvState implementation for native Candle Llama model KV cache management (Spec §9.2 Stufe B).
// INVARIANTEN: Zero-Panic doctrine (no .unwrap() or .expect() in production code);
// Full tensor export, import, and truncation for multi-layer KV caches.

use bytes::Bytes;
use candle_core::{DType, Device, Tensor};
use half::f16;
use contextra_types::ContextraError;
use contextra_ports::kv::KvBlock;
use serde::{Deserialize, Serialize};
use std::ops::Range;

/// Represents the key (`k`) and value (`v`) tensors for a single transformer layer.
#[derive(Debug, Clone)]
pub struct LayerKv {
    /// Key tensor of shape `(batch, seq_len, n_kv_head, head_dim)` or `(batch, n_kv_head, seq_len, head_dim)`.
    pub k: Tensor,
    /// Value tensor of shape `(batch, seq_len, n_kv_head, head_dim)` or `(batch, n_kv_head, seq_len, head_dim)`.
    pub v: Tensor,
}

impl LayerKv {
    /// Creates a new `LayerKv` pair.
    pub fn new(k: Tensor, v: Tensor) -> Self {
        Self { k, v }
    }
}

/// Explicit KV-Cache state holding layer-wise tensors and current sequence length position.
#[derive(Debug, Clone)]
pub struct KvState {
    /// Layer-wise KV cache tensor pairs.
    pub layers: Vec<LayerKv>,
    /// Active sequence length (token count) represented in the cache.
    pub pos: usize,
}

#[derive(Serialize, Deserialize)]
struct TensorPayload {
    shape: Vec<usize>,
    dtype: String,
    data: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
struct LayerPayload {
    k: TensorPayload,
    v: TensorPayload,
}

#[derive(Serialize, Deserialize)]
struct KvStateBlockPayload {
    layers: Vec<LayerPayload>,
    seq_len: usize,
}

fn tensor_to_payload(tensor: &Tensor) -> Result<TensorPayload, ContextraError> {
    let shape = tensor.dims().to_vec();
    let cpu_tensor = tensor
        .to_device(&Device::Cpu)
        .map_err(|e| ContextraError::Internal(format!("Failed to transfer tensor to CPU: {e}")))?;

    match tensor.dtype() {
        DType::F32 => {
            let flat = cpu_tensor
                .flatten_all()
                .map_err(|e| ContextraError::Internal(format!("Failed to flatten tensor: {e}")))?;
            let floats = flat.to_vec1::<f32>().map_err(|e| {
                ContextraError::Internal(format!("Failed to extract f32 vector: {e}"))
            })?;
            let data = floats.iter().flat_map(|f| f.to_le_bytes()).collect();
            Ok(TensorPayload {
                shape,
                dtype: "f32".to_string(),
                data,
            })
        }
        DType::F16 => {
            let flat = cpu_tensor
                .flatten_all()
                .map_err(|e| ContextraError::Internal(format!("Failed to flatten tensor: {e}")))?;
            let f16s = flat.to_vec1::<f16>().map_err(|e| {
                ContextraError::Internal(format!("Failed to extract f16 vector: {e}"))
            })?;
            let data = f16s.iter().flat_map(|f| f.to_le_bytes()).collect();
            Ok(TensorPayload {
                shape,
                dtype: "f16".to_string(),
                data,
            })
        }
        other => Err(ContextraError::capability_unsupported(
            "kv_tensor_serialization",
            format!("KV cache tensor serialization for dtype {other:?} is not supported"),
        )),
    }
}

fn payload_to_tensor(payload: &TensorPayload, device: &Device) -> Result<Tensor, ContextraError> {
    match payload.dtype.as_str() {
        "f32" => {
            if payload.data.len() % 4 != 0 {
                return Err(ContextraError::InvalidInput(
                    "Invalid f32 payload data length".to_string(),
                ));
            }
            let floats: Vec<f32> = payload
                .data
                .chunks_exact(4)
                .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
                .collect();

            let tensor = Tensor::from_vec(floats, payload.shape.as_slice(), &Device::Cpu)
                .map_err(|e| ContextraError::Internal(format!("Failed to build tensor: {e}")))?;
            tensor
                .to_device(device)
                .map_err(|e| ContextraError::Internal(format!("Failed to move tensor: {e}")))
        }
        "f16" => {
            if payload.data.len() % 2 != 0 {
                return Err(ContextraError::InvalidInput(
                    "Invalid f16 payload data length".to_string(),
                ));
            }
            let f16s: Vec<f16> = payload
                .data
                .chunks_exact(2)
                .map(|chunk| f16::from_le_bytes([chunk[0], chunk[1]]))
                .collect();

            let tensor = Tensor::from_vec(f16s, payload.shape.as_slice(), &Device::Cpu)
                .map_err(|e| ContextraError::Internal(format!("Failed to build tensor: {e}")))?;
            tensor
                .to_device(device)
                .map_err(|e| ContextraError::Internal(format!("Failed to move tensor: {e}")))
        }
        other => Err(ContextraError::capability_unsupported(
            "kv_tensor_deserialization",
            format!("Unsupported KV cache payload dtype: {other}"),
        )),
    }
}

impl KvState {
    /// Creates a new `KvState` wrapping layer-wise KV tensors and position tracking.
    pub fn new(layers: Vec<LayerKv>, pos: usize) -> Self {
        Self { layers, pos }
    }

    /// Returns `true` if there are no layer KV tensors or position is zero.
    pub fn is_empty(&self) -> bool {
        self.layers.is_empty() || self.pos == 0
    }

    /// Returns the active sequence length / position.
    pub fn pos(&self) -> usize {
        self.pos
    }

    /// Returns the number of transformer layers in this state.
    pub fn layer_count(&self) -> usize {
        self.layers.len()
    }

    fn tensor_seq_dim(tensor: &Tensor, pos: usize) -> usize {
        let dims = tensor.dims();
        if dims.len() == 4 {
            if dims[2] == pos {
                2
            } else if dims[1] == pos {
                1
            } else {
                2
            }
        } else if dims.len() == 2 {
            1
        } else {
            dims.len().saturating_sub(2)
        }
    }

    /// Truncates stored KV tensors to at most `len` tokens along the sequence dimension.
    pub fn truncate(&mut self, len: usize) -> Result<(), ContextraError> {
        if len >= self.pos {
            return Ok(());
        }

        for layer in self.layers.iter_mut() {
            if layer.k.dims().len() >= 2 {
                let dim = Self::tensor_seq_dim(&layer.k, self.pos);
                layer.k = layer.k.narrow(dim, 0, len).map_err(|e| {
                    ContextraError::Internal(format!("Failed to truncate key tensor: {e}"))
                })?;
            }
            if layer.v.dims().len() >= 2 {
                let dim = Self::tensor_seq_dim(&layer.v, self.pos);
                layer.v = layer.v.narrow(dim, 0, len).map_err(|e| {
                    ContextraError::Internal(format!("Failed to truncate value tensor: {e}"))
                })?;
            }
        }
        self.pos = len;
        Ok(())
    }

    /// Exports a sub-range of KV cache tensors across all layers into a `KvBlock`.
    pub fn export_block(&self, range: Range<usize>) -> Result<KvBlock, ContextraError> {
        if range.start > range.end || range.end > self.pos {
            return Err(ContextraError::InvalidInput(format!(
                "Range {:?} is out of bounds for KvState pos {}",
                range, self.pos
            )));
        }

        let len = range.end - range.start;
        let mut layer_payloads = Vec::with_capacity(self.layers.len());

        for layer in &self.layers {
            let dim_k = Self::tensor_seq_dim(&layer.k, self.pos);
            let dim_v = Self::tensor_seq_dim(&layer.v, self.pos);

            let k_slice = layer
                .k
                .narrow(dim_k, range.start, len)
                .map_err(|e| ContextraError::Internal(format!("Failed to slice key tensor: {e}")))?;
            let v_slice = layer.v.narrow(dim_v, range.start, len).map_err(|e| {
                ContextraError::Internal(format!("Failed to slice value tensor: {e}"))
            })?;

            let k_payload = tensor_to_payload(&k_slice)?;
            let v_payload = tensor_to_payload(&v_slice)?;

            layer_payloads.push(LayerPayload {
                k: k_payload,
                v: v_payload,
            });
        }

        let payload = KvStateBlockPayload {
            layers: layer_payloads,
            seq_len: len,
        };

        let encoded = bincode::serialize(&payload).map_err(|e| {
            ContextraError::Internal(format!("Failed to serialize KvState block payload: {e}"))
        })?;

        Ok(KvBlock {
            block_id: range.start as u64,
            data: Bytes::from(encoded),
        })
    }

    /// Imports a `KvBlock` and concatenates its layer tensors onto the current state.
    pub fn import_block(&mut self, block: &KvBlock) -> Result<(), ContextraError> {
        let payload: KvStateBlockPayload = bincode::deserialize(&block.data).map_err(|e| {
            ContextraError::InvalidInput(format!("Failed to deserialize KvBlock payload: {e}"))
        })?;

        let device = if let Some(first_layer) = self.layers.first() {
            first_layer.k.device().clone()
        } else {
            Device::Cpu
        };

        if self.layers.is_empty() {
            for lp in payload.layers {
                let k = payload_to_tensor(&lp.k, &device)?;
                let v = payload_to_tensor(&lp.v, &device)?;
                self.layers.push(LayerKv::new(k, v));
            }
            self.pos = payload.seq_len;
        } else {
            if self.layers.len() != payload.layers.len() {
                return Err(ContextraError::InvalidInput(format!(
                    "Layer count mismatch: state has {} layers, block has {}",
                    self.layers.len(),
                    payload.layers.len()
                )));
            }

            let cur_pos = self.pos;
            for (layer, lp) in self.layers.iter_mut().zip(payload.layers) {
                let imp_k = payload_to_tensor(&lp.k, &device)?;
                let imp_v = payload_to_tensor(&lp.v, &device)?;

                let dim_k = Self::tensor_seq_dim(&layer.k, cur_pos);
                let dim_v = Self::tensor_seq_dim(&layer.v, cur_pos);

                layer.k = Tensor::cat(&[&layer.k, &imp_k], dim_k).map_err(|e| {
                    ContextraError::Internal(format!("Failed to concatenate key tensors: {e}"))
                })?;
                layer.v = Tensor::cat(&[&layer.v, &imp_v], dim_v).map_err(|e| {
                    ContextraError::Internal(format!("Failed to concatenate value tensors: {e}"))
                })?;
            }
            self.pos += payload.seq_len;
        }

        Ok(())
    }

    /// Imports a `KvBlock` at a specific position or appends it if `at == self.pos`.
    pub fn import_block_at(&mut self, block: &KvBlock, at: usize) -> Result<(), ContextraError> {
        if at == self.pos {
            self.import_block(block)
        } else if at < self.pos {
            self.truncate(at)?;
            self.import_block(block)
        } else {
            Err(ContextraError::InvalidInput(format!(
                "Cannot import block at position {at} past current pos {}",
                self.pos
            )))
        }
    }
}
