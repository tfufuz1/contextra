//! Observability traits and re-exports for memory lifecycle and grounding validation.

// FILE-CONTEXT
// STAND: 2026-09-15T00:00:00Z
// ZWECK: Observability-Modul für Grounding-Validierung.
// INVARIANTEN: Re-exportiert primäre Grounding Trait-Definitionen aus `lifecycle`.

pub use super::lifecycle::{GroundingAssessment, GroundingValidator, ResponseGroundingValidator};

/// Trait for querying Lyapunov drift status from an attached router engine without creating a cyclic dependency (ADR-080).
///
/// | Trait | Produktiv-Impl | Composition-Root-Aufrufer | Status |
/// |---|---|---|---|
/// | `DriftStatusProvider` | `EnsembleDriftWatcher` (`contextra-adapt`) / `RouterDriftAdapter` (`contextra-ports`) | `contextra-router` | Aktiver Port im Router-Subsystem |
pub trait DriftStatusProvider: Send + Sync {
    /// Returns the overall drift status string ("stabil", "warnung", "kritisch", or "unbekannt").
    fn overall_drift_status(&self) -> String;
}
