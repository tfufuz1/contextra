//! Trait für Storage- und Index-Scanner zur Restmengen-Messung.

#![forbid(unsafe_code)]

use crate::error::{ScanError, SpiResult};
use crate::layer::{ScanTechnique, WitnessLayer};
use crate::scope::ScanTarget;
use crate::session::ScanSession;
use crate::witness::LayerWitness;
use contextra_types::RequestContext;

/// Trait für Storage-Scanner zur Durchführung von Restmengen-Messungen.
///
/// Scanner implementieren `scan_units` zur Erfassung gescannter Einheiten
/// und Verifizierung der Positivkontrolle auf der übergebenen `ScanSession`.
pub trait LayerScanner: Send + Sync {
    /// Gibt die von diesem Scanner bediente Schicht zurück.
    fn layer(&self) -> WitnessLayer;

    /// Führt das Scannen von Einheiten auf der übergebenen `ScanSession` aus.
    fn scan_units(&self, session: &mut ScanSession, ctx: &RequestContext) -> Result<(), ScanError>;

    /// Führt die vollständige Messung aus (`begin`, `scan_units`, `finish`).
    fn scan(
        &self,
        ctx: &RequestContext,
        target: ScanTarget,
        technique: ScanTechnique,
        now_unix_nanos: u64,
    ) -> SpiResult<LayerWitness> {
        let mut session = ScanSession::begin(self.layer(), target, technique);
        if let Err(e) = self.scan_units(&mut session, ctx) {
            return Err(crate::error::SpiError::Internal(e.to_string()));
        }
        session.finish(now_unix_nanos)
    }
}
