//! Core domain, budget, filter, importance, and search query data types.

/// Memory and token resource budget management types.
pub mod budget;
/// Fundamental domain models (`DocId`, `TxId`, `Entity`, `DistanceMetric`, etc.).
pub mod domain;
/// Structured metadata expression filter types.
pub mod filter;
/// Memory importance and recency decay scoring types.
pub mod importance;
/// Math utility types for safe arithmetic and saturating casts.
pub mod math;
/// Unified 4-signal search query and context types.
pub mod saos;

pub use budget::*;
#[allow(ambiguous_glob_reexports)]
pub use domain::*;
pub use filter::*;
pub use importance::*;
pub use math::*;
#[allow(ambiguous_glob_reexports)]
pub use saos::*;

/// Branchless saturating narrowing integer conversions.
pub mod saturating;
pub use saturating::*;
