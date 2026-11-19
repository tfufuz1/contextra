//! `ErasureLayer` nach SPEC §7. Implementiert von E1-Crates, definiert in E0.

use std::future::Future;
use std::pin::Pin;

use crate::error::ErasureError;
use crate::types_local::{Armed, KeyCoverage, LayerId, Scanner};

use super::target::EraseTarget;

/// Boxed Future für objektsichere Layer-Methoden.
pub type LayerFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T, ErasureError>> + Send + 'a>>;

/// Geltungsbereich einer Schicht für ein Ziel (von `resolve` geliefert).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayerScope {
    /// Stabile Kennung des Bereichs.
    pub scope_id: [u8; 16],
    /// Betroffene Einheiten (Segmente, Knoten, Slots, …).
    pub affected_units: Vec<u64>,
}

/// Kontext eines Purge-Aufrufs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PurgeCx {
    /// Frist in Millisekunden seit Epoch.
    pub deadline_ms: u64,
}

/// Ergebnis eines Purge-Aufrufs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PurgeOutcome {
    /// Anzahl bereinigter Einheiten.
    pub purged_units: u64,
}

/// Schnittstelle für Layer-Implementierer (SPEC §7).
///
/// Invarianten für Implementierer: `purge` ist idempotent (zweiter Aufruf ist ein No-op),
/// `witness` ist rein lesend, Logs enthalten nie Klartext des Ziels (INV-12).
pub trait ErasureLayer: Send + Sync {
    /// Schichtkennung.
    fn id(&self) -> LayerId;

    /// Schlüsselabdeckung dieser Schicht.
    fn key_coverage(&self) -> KeyCoverage;

    /// Löst das Ziel in den konkreten Geltungsbereich dieser Schicht auf.
    fn resolve<'a>(&'a self, target: &'a EraseTarget) -> LayerFuture<'a, LayerScope>;

    /// Vorbereitung im RAM nach dem Gate (Surrogate markieren, Filter befüllen).
    fn on_gate<'a>(&'a self, scope: &'a LayerScope) -> LayerFuture<'a, ()>;

    /// Physische Bereinigung. Idempotent.
    fn purge<'a>(&'a self, scope: &'a LayerScope, cx: &'a PurgeCx)
        -> LayerFuture<'a, PurgeOutcome>;

    /// Rein lesende Rückstandsprüfung. Schreibt Treffer in den Scanner und liefert die Trefferzahl.
    fn witness<'a>(
        &'a self,
        scope: &'a LayerScope,
        scanner: &'a mut Scanner<Armed>,
    ) -> LayerFuture<'a, u64>;
}
