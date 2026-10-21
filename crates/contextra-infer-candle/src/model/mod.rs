// FILE-CONTEXT
// STAND: 2026-10-21T00:00:00Z
// ZWECK: Forked Llama and Qwen2 model modules with direct KvState access (Spec §9.2 Stufe B).
// INVARIANTEN: Zero-Panic doctrine (no .unwrap() or .expect() in production code).

pub mod quantized_llama;
pub mod quantized_qwen2;

pub use quantized_llama::{LayerWeights, ModelWeights};
