//! # Modul: mod.rs
//!
//! **Zweck:** Suchsubsystem fuer Collections (Text-, Vektor- und Hybridsuche).
//!
//! **Architektur:** Gehoert zur Crate `contextra-engine` (Ring 2).
//! Eingehaltene Invarianten: Fail-Closed bei Fehlern, strikte Mandanten- und Indexisolation.

mod basic;
mod filtered;
mod hybrid;
mod hydrate;

pub use hydrate::apply_read_gate;

use super::{extract_effective_importance, Collection, StoredDocument, StoredDocumentMeta};
pub use crate::temporal_filter::{
    apply_temporal_validity_filter, apply_temporal_validity_filter_at, FusionResult,
};
