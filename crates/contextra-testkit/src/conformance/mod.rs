//! Conformance test suites for Port traits.
//!
//! Provides reusable contract verification functions for adapter implementations across Contextra.
//!
//! # Overview
//! Each suite function accepts a factory closure or an instance reference, executes a series of
//! rigorous contract verification tests, and returns a [`ConformanceResult`]. On failure,
//! a detailed [`ConformanceFailure`] is returned without panicking in the library itself.
//! Downstream test runners can convert failures into test panics using [`assert_conformance!`].
//!
//! # Usage Example
//!
//! ```ignore
//! #[cfg(test)]
//! mod tests {
//!     use contextra_testkit::assert_conformance;
//!     use contextra_testkit::conformance::*;
//!
//!     #[tokio::test]
//!     async fn test_my_storage_conformance() {
//!         let res = run_storage_engine_suite(|| async { MyStorageEngine::new() }).await;
//!         assert_conformance!(res);
//!     }
//! }
//! ```

pub mod clock;
pub mod failure;
pub mod graph;
pub mod macros;
pub mod metrics;
pub mod rng;
pub mod storage;
pub mod text;
pub mod vector;

pub use clock::run_clock_suite;
pub use failure::{ConformanceFailure, ConformanceResult};
pub use graph::run_graph_index_suite;
pub use metrics::run_metrics_sink_suite;
pub use rng::run_rng_suite;
pub use storage::run_storage_engine_suite;
pub use text::run_text_index_suite;
pub use vector::run_vector_index_suite;
