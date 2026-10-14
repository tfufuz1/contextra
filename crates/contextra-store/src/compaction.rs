//! Background compaction engine for the LSM-Tree.

pub mod adaptive;
mod config;
mod engine;
pub mod merge_operator;
pub mod purge_filter;
pub mod retention;
pub mod wiring;

#[cfg(test)]
mod tests;

pub use adaptive::*;
pub use config::*;
pub use engine::*;
pub use merge_operator::*;
pub use purge_filter::*;
pub use retention::*;
pub use wiring::*;

/// Removed TTL feature stub per Decision Option B.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TtlMetadata;
