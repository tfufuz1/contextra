//! Erase-Observer port trait and event representations (SPEC §3.1, §8.2, D-118, T-OBS-1..3).
//!
//! # Normative Referenz
//! - **D-118:** Zustellung von Löschereignissen an Betreiber-Hooks.
//! - **P13:** Das Ereignis enthält `ErasureId`, Zieltyp, Phase und Zeit.
//!   Nie Commitment, Betroffenenkennung oder Inhalt.
//! - Zustellung erfolgt entkoppelt; observer darf den Löschpfad nicht blockieren oder abbrechen.

use crate::BoxFuture;
use contextra_types::ids::ErasureId;
use std::fmt;
use std::sync::Arc;

/// Type representing a millisecond or microsecond timestamp (SPEC D-118).
pub type Timestamp = u64;

/// Target kind indicating which hierarchy level was targeted (SPEC D-118, P13).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TargetKind {
    /// Single document erasure.
    Document,
    /// Subject-level erasure (all documents belonging to a subject).
    Subject,
    /// Collection-level erasure.
    Collection,
    /// Tenant-level erasure.
    Tenant,
    /// Document replacement (`replace_erasing`, D-115).
    DocumentReplace,
    /// Superseded document version (`update_with(..., Purge)`, D-117).
    Superseded,
}

/// Observed lifecycle phase of an erasure operation (SPEC D-118).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ObservedPhase {
    /// Read-gate is established; all future reads will see target as erased.
    Gated,
    /// Physical purge and verification complete.
    Completed,
    /// Access restriction per Art. 18 DSGVO is activated (D-116).
    Restricted,
    /// Access restriction is lifted (D-116).
    Lifted,
}

/// Event published to an [`ErasureObserver`] upon reaching an erasure lifecycle phase (SPEC D-118).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErasureEvent {
    /// Unique identifier of the erasure operation.
    pub id: ErasureId,
    /// Kind of target affected.
    pub target_kind: TargetKind,
    /// Current phase reached.
    pub phase: ObservedPhase,
    /// UTC timestamp of event generation.
    pub at: Timestamp,
}

/// Error returned by an [`ErasureObserver`] during notification delivery (SPEC D-118).
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ObserverError {
    /// Delivery failed due to transient network or backend error.
    #[error("transient delivery failure: {0}")]
    Transient(String),

    /// Delivery rejected permanently by the receiver.
    #[error("permanent delivery failure: {0}")]
    Permanent(String),

    /// Notification timed out after the maximum allowed duration (30 s per SPEC D-118).
    #[error("observer delivery timed out")]
    Timeout,
}

/// Port trait for asynchronous delivery of erasure notifications (SPEC §3.1, §8.2, D-118).
///
/// Implementations must be object-safe and dyn-compatible as `Arc<dyn ErasureObserver>`.
pub trait ErasureObserver: Send + Sync + 'static {
    /// Notifies the observer about an erasure event.
    ///
    /// # Fehler
    /// Liefert [`ObserverError`], wenn die Benachrichtigung scheitert. Die aufrufende
    /// Hintergrundaufgabe führt Wiederholungen mit exponentiellem Backoff durch (D-118).
    ///
    /// # Crash
    /// Quittungen (`ObserverAck`) werden erst nach erfolgreicher Rückgabe im Journal
    /// persistiert; nicht quittierte Ereignisse werden nach Neustart erneut zugestellt.
    fn on_erased(
        &self,
        ev: ErasureEvent,
    ) -> BoxFuture<'static, std::result::Result<(), ObserverError>>;
}

impl<T: ErasureObserver + ?Sized> ErasureObserver for Arc<T> {
    fn on_erased(
        &self,
        ev: ErasureEvent,
    ) -> BoxFuture<'static, std::result::Result<(), ObserverError>> {
        (**self).on_erased(ev)
    }
}

impl fmt::Display for TargetKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Document => write!(f, "Document"),
            Self::Subject => write!(f, "Subject"),
            Self::Collection => write!(f, "Collection"),
            Self::Tenant => write!(f, "Tenant"),
            Self::DocumentReplace => write!(f, "DocumentReplace"),
            Self::Superseded => write!(f, "Superseded"),
        }
    }
}

impl fmt::Display for ObservedPhase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Gated => write!(f, "Gated"),
            Self::Completed => write!(f, "Completed"),
            Self::Restricted => write!(f, "Restricted"),
            Self::Lifted => write!(f, "Lifted"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::FutureExt;

    struct MockObserver;

    impl ErasureObserver for MockObserver {
        fn on_erased(
            &self,
            _ev: ErasureEvent,
        ) -> BoxFuture<'static, std::result::Result<(), ObserverError>> {
            Box::pin(async move { Ok(()) })
        }
    }

    /// SPEC: T-OBS-1
    #[test]
    fn test_dyn_erasure_observer() {
        let obs: Arc<dyn ErasureObserver> = Arc::new(MockObserver);
        let ev = ErasureEvent {
            id: ErasureId::new([42u8; 16]),
            target_kind: TargetKind::Document,
            phase: ObservedPhase::Gated,
            at: 1000,
        };
        let res = obs.on_erased(ev).now_or_never().expect("now or never");
        assert!(res.is_ok());
    }
}
