//! # WAL Modultests (Ring 1)
//!
//! Sammelmodul fuer interne WAL-Tests.

use super::KeyManager;
pub use super::*;
use contextra_types::{ContextraError, Result};

#[cfg(test)]
mod encode_tests;
#[cfg(test)]
mod flusher_tests;
#[cfg(test)]
mod hmac_tests;
#[cfg(test)]
mod io_tests;
#[cfg(test)]
mod replay_tests;
