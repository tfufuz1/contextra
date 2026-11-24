//! Synchrones Scope-Purge für WAL-Einträge (SPEC §11, WP-7, INV-ERA-6).
//!
//! Stellt die vollständige Entsorgung von WAL-Einträgen sicher:
//! Bevorzugt physische Tilgung (Truncate/Overwrite), andernfalls Unlesbarmachung durch
//! Vernichtung des scope-spezifischen Integritäts-/Kryptoschlüssels.
//!
//! // SECTION ScopePurge
//! // ANCHOR[contextra-erasure:SCOPE-PURGE-001]

use super::wal_sink::{WalDisposalMethod, WalDisposalReceipt, WalScope, WalSink, WalSinkError};

#[cfg(not(test))]
use crate::types_local::Clock;
#[cfg(test)]
use contextra_erasure::types_local::Clock;

/// Führt die WAL-Entsorgung für den angegebenen Scope synchron durch.
///
/// Gemäß INV-ERA-6 werden WAL-Einträge eines gelöschten Scopes entsorgt oder unlesbar gemacht.
/// - **Wiederaufnehmbarkeit & Idempotenz (INV-ERA-2):** Wurde der Scope bereits entsorgt, wird
///   dies sofort erkannt und quittiert (`WalDisposalMethod::AlreadyDisposed`).
/// - **Priorität:** Zuerst werden alle auffindbaren Einträge physisch getilgt (`dispose_entries`).
/// - **Sicherheitsfall:** Anschließend bzw. zur vollständigen Entwertung wird der scope-spezifische
///   Integritäts-/Kryptoschlüssel über `discard_scope_integrity_key` vernichtet, sodass auch
///   etwaige Restfragmente unwiederbringlich unlesbar werden.
/// - **Fail-Closed:** Tritt während einer Phase ein Fehler auf, wird die Löschung als NICHT
///   vollendet gemeldet und ein präziser `WalSinkError` propagiert.
///
/// Die Ledger-Verankerung erfolgt auf höherer Orchestratorebene (P10).
pub fn dispose_scope_wal(
    sink: &dyn WalSink,
    scope: &WalScope,
    clock: &dyn Clock,
) -> Result<WalDisposalReceipt, WalSinkError> {
    // 1. Idempotenz- und Wiederaufnahmeprüfung
    if sink.is_scope_disposed(scope)? {
        let timestamp_nanos = clock.now_unix_nanos();
        return Ok(WalDisposalReceipt {
            scope: scope.clone(),
            disposed_entries_count: 0,
            method: WalDisposalMethod::AlreadyDisposed,
            timestamp_nanos,
        });
    }

    // 2. Einträge des Scopes enumerieren
    let entries = sink.enumerate_scope_entries(scope)?;

    let mut disposed_count = 0;
    let mut had_entries = false;

    // 3. Bevorzugt physische Tilgung der Einträge
    if !entries.is_empty() {
        disposed_count = sink.dispose_entries(scope, &entries)?;
        had_entries = true;
    }

    // 4. Scope-Integritäts-/Kryptoschlüssel verwerfen (Garantie der Nicht-Wiederherstellbarkeit)
    sink.discard_scope_integrity_key(scope)?;

    let method = if had_entries || disposed_count > 0 {
        WalDisposalMethod::PhysicalPurge
    } else {
        WalDisposalMethod::CryptoShredding
    };

    // 5. Zeitstempel über den Clock-Port ermitteln
    let timestamp_nanos = clock.now_unix_nanos();

    // 6. Quittung / Beleg zurückgeben
    Ok(WalDisposalReceipt {
        scope: scope.clone(),
        disposed_entries_count: disposed_count,
        method,
        timestamp_nanos,
    })
}
// !SECTION
