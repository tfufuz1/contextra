//! # Modul: fixtures.rs
//!
//! **Zweck:** Test-Fixtures und Hilfsdaten für Collection-Tests.
//!
//! **Architektur:** Gehört zur Crate `contextra-engine` (Ring 2).
//! Eingehaltene Invarianten: Fail-Closed bei Fehlern, strikte Mandanten- und Indexisolation.

// Common fixtures, imports and helper functions shared across collection test modules.

pub(super) use crate::collection::Collection;
