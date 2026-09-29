//! Background compaction engine for the LSM-Tree.

pub mod adaptive;
mod config;
mod engine;
pub mod merge_operator;
pub mod retention;

#[cfg(test)]
mod tests;

pub use adaptive::*;
pub use config::*;
pub use engine::*;
pub use merge_operator::*;
pub use retention::*;

/// Removed TTL feature stub per Decision Option B.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TtlMetadata;
