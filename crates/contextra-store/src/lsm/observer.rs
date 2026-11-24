//! # WAL-Observer & Ereignisbeobachter (Ring 1)
//!
//! // FILE-CONTEXT
//! // STAND: 2026-11-11
//! // ZWECK: Fassade fuer das WalObserver-Subsystem (Typen, Registry, Adapter, Worker)
//! // INVARIANTEN: Fail-open Semantik, Circuit-Breaker, Non-blocking Commit
//!
//! Informiert registrierte Beobachter ueber persistierte WAL-Eintraege.

mod adapter;
mod registry;
mod types;
mod worker;

pub use adapter::AsyncObserverAdapter;
pub use registry::ObserverRegistry;
pub use types::{
    CommitContext, CommittedBatch, WalEntryRef, WalObserver, WriteOrigin,
    DEFAULT_MAX_OBSERVER_LATENCY,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_origin_user_write() {
        let origin = WriteOrigin::UserWrite;
        assert_eq!(origin, WriteOrigin::UserWrite);
    }
}
