//! # LSM-Kernoperationen (Ring 1)
//!
//! Buendelt Lese-, Schreib-, Wartungs- und Kompaktierungsoperationen.

pub(crate) mod compaction;
pub(crate) mod maintenance;
pub(crate) mod read;
pub(crate) mod write;

use super::*;

// MIGRATION(S-70): neuer Pfad crate::compat::engine::lsm
#[cfg(feature = "workspace")]
pub use crate::compat::engine::*;
