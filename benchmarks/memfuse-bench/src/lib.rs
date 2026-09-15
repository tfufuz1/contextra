// FILE-CONTEXT
// STAND: 2026-09-07
// ZWECK: Exportierte Module für benchmarks/memfuse-bench

#[cfg(feature = "external-benchmarks")]
pub mod ann_benchmarks;
#[cfg(feature = "external-benchmarks")]
pub mod beir_eval;
pub mod compare;
pub mod locomo;
pub mod long_mem_eval;
pub mod path_rag_sweep;
pub mod regression_gate;
