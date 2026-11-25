// REVIEW-PASS[1/2] (TS: 2026-09-16T16:20:00Z) (SESSION: 11638515275805963653) PRÜFER-KONTEXT: FRESH
// FILE-CONTEXT
// STAND: 2026-09-16
// ZWECK: Exportierte Module für benchmarks/contextra-bench

#![forbid(unsafe_code)]

#[cfg(feature = "external-benchmarks")]
pub mod ann_benchmarks;
#[cfg(feature = "external-benchmarks")]
pub mod beir_eval;
pub mod compare;
pub mod locomo;
pub mod long_mem_eval;
pub mod path_rag_sweep;
pub mod regression_gate;
