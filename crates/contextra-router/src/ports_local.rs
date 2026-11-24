//! # Lokale Port-Re-Exports (Veraltet)
//!
//! **Ring:** Ring 3 (Applikations- und Koordinationsschicht).
//! **Zweck:** Rückwärtskompatible Re-Exports für Port-Traits, die im Rahmen der Architektur-Bereinigung
//! (Ring-Modell Welle 2) vollständig in das Crate `contextra-ports` (Ring 0) überführt wurden.
//!
//! **Abhängigkeiten:**
//! - Upstream: `contextra_ports`.
//! - Downstream: Veraltete Aufrufer, die noch auf `contextra_router::ports_local` verweisen.

// ANCHOR[contextra-router:PRT-001]
#[deprecated(
    since = "0.1.0",
    note = "Moved to contextra_ports as part of Ring-Modell Welle 2 (siehe docs/refactor/router-db-edge-audit.md)"
)]
pub use contextra_ports::{
    CommunityResolver, ContextChunk, ContextPreparer, ContextWindow, DriftStatusProvider,
    HybridSearchProvider, TokenBudget,
};

#[cfg(test)]
mod tests {
    #[allow(deprecated)]
    use super::*;

    // ANCHOR[contextra-router:PRT-T01]
    // Prüft zur Compile-Zeit, dass die re-exportierten Typen die Trait-Signaturen aus contextra-ports erfüllen.
    #[test]
    fn test_ports_local_reexports() {
        fn _assert_community_resolver<T: CommunityResolver + ?Sized>() {}
        fn _assert_context_preparer<T: ContextPreparer + ?Sized>() {}
        fn _assert_hybrid_search_provider<T: HybridSearchProvider + ?Sized>() {}
        fn _assert_drift_status_provider<T: DriftStatusProvider + ?Sized>() {}
    }
}
