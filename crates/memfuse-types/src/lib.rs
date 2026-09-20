//! `MemFuse` Types — Canonical domain models, IDs, budgets, filters, and error types.
//!
//! # Architecture Role (Ring 0)
//!
//! This crate defines the foundational types and errors for the entire `MemFuse` system.
//! In accordance with the Ring-0 architecture, this crate has NO asynchronous runtimes
//! and NO I/O dependencies.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod error;
pub mod error_dto;
pub mod model_fingerprint;
pub mod schema;
pub mod tombstone;
pub mod types;

pub use error::{MemFuseError, Result};
pub use error_dto::MemFuseErrorDto;
pub use model_fingerprint::ModelFingerprint;
pub use schema::{DocIdWidth, ManifestSchemaVersion};
pub use tombstone::{SeqBitTombstone, TombstoneSemanticsCheck};
pub use types::*;
