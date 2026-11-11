#![forbid(unsafe_code)]

pub mod adapters;
#[cfg(feature = "external-benchmarks")]
pub mod ann_benchmarks;
#[cfg(feature = "external-benchmarks")]
pub mod beir_eval;
pub mod compare;
pub mod matrix;
pub mod metrics;
#[cfg(feature = "datasets")]
pub mod path_rag_sweep;
pub mod regression_gate;
