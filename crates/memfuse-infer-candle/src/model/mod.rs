// FILE-CONTEXT
// STAND: 2026-09-22T00:00:00Z
// ZWECK: Forked Llama model module with direct KvState access (Spec §9.2 Stufe B).
// INVARIANTEN: Zero-Panic doctrine (no .unwrap() or .expect() in production code).

pub mod quantized_llama;

pub use quantized_llama::{ModelWeights, LayerWeights};
