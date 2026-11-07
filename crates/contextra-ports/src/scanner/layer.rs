// FILE-CONTEXT
// ZWECK: Definition der Schichten, Scantechniken, Target-Deskriptoren und Purger-Traits (Vertrag V1, §12.7.4).
// HINWEIS: Typen sind nach contextra-erasure-spi migriert (Entscheidung D-E1).

#![forbid(unsafe_code)]

pub use contextra_erasure_spi::layer::{PurgeStats, ScanTechnique, WitnessLayer};
pub use contextra_erasure_spi::scope::ScanTarget;

use crate::Result;

/// Trait für physische Bereinigungsaktionen (Purger) einer Schicht.
///
/// Zur Abwärtskompatibilität in contextra-ports mit RequestContext und Result.
pub trait Purger: Send + Sync {
    /// Führt die physische Bereinigung für das gegebene Ziel aus.
    fn purge(&self, ctx: &crate::RequestContext, target: &ScanTarget) -> Result<PurgeStats>;
}
