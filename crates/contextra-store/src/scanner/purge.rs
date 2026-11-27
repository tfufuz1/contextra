//! # Purge-Implementierung (Ring 1) — gesperrt (WP-0)
//!
//! `LsmPurger` ist **nicht lauffähig**. Der bisherige Rewrite wertete das Purge-Ziel nicht
//! aus und konnte fremde Datensätze verwerfen, verschlüsselte Speicher übersehen und trotzdem
//! Erfolg melden (K2 der Löschketten-RFC, §4.2 / §6.1).
//!
//! Die Implementierung wird in WP-4 durch einen prädikatsgefilterten Rewrite über den
//! Compaction-Builder ersetzt. Bis dahin meldet jeder Aufruf einen Fehler (fail-closed).
//! Der alte Rewrite ist über die Git-Historie erreichbar.

// FILE-CONTEXT
// ZWECK: Platzhalter des physischen Purgers für LSM Storage bis WP-4 der Löschketten-RFC.
// INVARIANTEN: Keine Schreibzugriffe auf Dateien. Jeder Aufruf liefert `Err`.

#![forbid(unsafe_code)]

use super::runtime::ScannerRuntime;
use crate::lsm::LsmStorage;
use contextra_ports::scanner::{PurgeStats, Purger, ScanTarget};
use contextra_types::{ContextraError, RequestContext, Result};
use std::sync::Arc;

/// Purger für LSM Storage Engine. Gesperrt, siehe Modul-Dokumentation.
pub struct LsmPurger {
    // Felder bleiben für den Ersatz in WP-4 erhalten.
    #[allow(dead_code)]
    storage: Arc<LsmStorage>,
    #[allow(dead_code)]
    runtime: ScannerRuntime,
}

impl LsmPurger {
    /// Erzeugt einen neuen `LsmPurger` mit implizit erfasster Laufzeit.
    pub fn new(storage: Arc<LsmStorage>) -> Self {
        Self::with_runtime(storage, ScannerRuntime::capture())
    }

    /// Erzeugt einen neuen `LsmPurger` mit expliziter `ScannerRuntime`.
    pub fn with_runtime(storage: Arc<LsmStorage>, runtime: ScannerRuntime) -> Self {
        Self { storage, runtime }
    }
}

impl Purger for LsmPurger {
    fn purge(&self, _ctx: &RequestContext, _target: &ScanTarget) -> Result<PurgeStats> {
        Err(ContextraError::Storage(
            "LsmPurger gesperrt: Purge ist nicht unterstützt, bis der prädikatsgefilterte \
             Rewrite (RFC WP-4) umgesetzt ist"
                .to_string(),
        ))
    }
}
