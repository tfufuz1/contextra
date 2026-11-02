//! Benchmark-Harness und Regressionsprüfung für Contextra-Crates.

pub mod harness;
pub mod regression;

pub use harness::{BenchmarkHarness, BenchmarkResult, HardwareMetadata};
pub use regression::{check_regression, BaselineReport, RegressionConfig, RegressionError};
