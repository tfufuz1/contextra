//! # Scan-Tests (Ring 1)
//!
//! Testet Bereichsabfragen ueber gemischte Speicherbereiche.
//! Zerlegt in:
//! - [`bounded`]: Begrenzte Scans und Paginierung
//! - [`prefix`]: MVCC-Snapshot-Isolation und Praefix-Filterung
//! - [`prop`]: Eigenschaftsbasierte Tests

pub mod bounded;
pub mod prefix;
pub mod prop;

use super::*;
