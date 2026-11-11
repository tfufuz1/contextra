//! # Recovery-Tests (Ring 1)
//!
//! Validiert Neustarts nach simulierten Systemabstuerzen.
//! Zerlegt in:
//! - [`rollback`]: Transaktions-Rollback und SSTable-Rekompaktierung
//! - [`startup`]: Startup-Replay, Uncommitted/Aborted-Transaktionsbereinigung und Discovery

pub mod rollback;
pub mod startup;

use super::*;
use std::sync::atomic::Ordering;
use tempfile::TempDir;
