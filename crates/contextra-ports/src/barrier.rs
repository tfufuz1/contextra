//! Synchronization barrier ports for concurrency control between writes, compactions, and purges.
//!
//! # SPEC-REF (L-7, SPEC §11.3, §11.4, D-57, D-78)
//! - [`CommitBarrier`]: Controls synchronization between concurrent group commits (shared)
//!   and erasure gate transitions (exclusive).
//! - [`PurgeBarrier`]: Controls mutual exclusion between background LSM compactions (shared)
//!   and physical purges (exclusive).
//!
//! # Invariants
//! All guard types ([`BarrierShared`], [`BarrierExclusive`], [`CompactionPermit`], [`PurgePermit`])
//! are owned token types without lifetime borrows, ensuring cancel-safety and avoiding
//! holding locks across alien `.await` points.

use crate::BoxFuture;
use std::fmt;
use std::sync::Arc;

/// RAII token guard representing a shared commit phase permit.
pub struct BarrierShared {
    _token: Option<Arc<()>>,
}

impl BarrierShared {
    /// Creates a new `BarrierShared` token.
    pub fn new(token: Option<Arc<()>>) -> Self {
        Self { _token: token }
    }
}

impl fmt::Debug for BarrierShared {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BarrierShared")
    }
}

/// RAII token guard representing an exclusive gate phase permit.
pub struct BarrierExclusive {
    _token: Option<Arc<()>>,
}

impl BarrierExclusive {
    /// Creates a new `BarrierExclusive` token.
    pub fn new(token: Option<Arc<()>>) -> Self {
        Self { _token: token }
    }
}

impl fmt::Debug for BarrierExclusive {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BarrierExclusive")
    }
}

/// RAII token guard representing a compaction permit.
pub struct CompactionPermit {
    _token: Option<Arc<()>>,
}

impl CompactionPermit {
    /// Creates a new `CompactionPermit` token.
    pub fn new(token: Option<Arc<()>>) -> Self {
        Self { _token: token }
    }
}

impl fmt::Debug for CompactionPermit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CompactionPermit")
    }
}

/// RAII token guard representing an exclusive purge permit.
pub struct PurgePermit {
    _token: Option<Arc<()>>,
}

impl PurgePermit {
    /// Creates a new `PurgePermit` token.
    pub fn new(token: Option<Arc<()>>) -> Self {
        Self { _token: token }
    }
}

impl fmt::Debug for PurgePermit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PurgePermit")
    }
}

/// Barrier trait managing commit phases and exclusive gate operations (SPEC §11.3, D-57, D-78, L-7).
pub trait CommitBarrier: Send + Sync {
    /// Acquires a shared permit for standard group commit operations.
    ///
    /// # Fehler
    /// Unfehlbar; wartet asynchron auf Commit-Freigabe.
    ///
    /// # Crash
    /// Reines Synchronisations-Token im RAM; cancel-safe.
    fn shared(&self) -> BoxFuture<'_, BarrierShared>;

    /// Acquires an exclusive permit for gate group transitions.
    ///
    /// # Fehler
    /// Unfehlbar; blockiert nachfolgende Commits exklusiv.
    ///
    /// # Crash
    /// Exklusiver Token-Inhaber; cancel-safe.
    fn exclusive(&self) -> BoxFuture<'_, BarrierExclusive>;
}

/// Barrier trait managing LSM compaction and physical purge mutual exclusion (SPEC §11.3, §11.4, I-16, L-7).
pub trait PurgeBarrier: Send + Sync {
    /// Acquires a shared permit for compaction.
    ///
    /// # Fehler
    /// Unfehlbar; wartet auf Purge-Freigabe.
    ///
    /// # Crash
    /// Kooperative Kompaktions-Sperre im RAM.
    fn compaction(&self) -> BoxFuture<'_, CompactionPermit>;

    /// Acquires an exclusive permit for physical purging.
    ///
    /// # Fehler
    /// Unfehlbar; exklusiver Purge-Zugriff.
    ///
    /// # Crash
    /// Erzwingt Abbruch kooperativer Kompaktionen; cancel-safe.
    fn purge(&self) -> BoxFuture<'_, PurgePermit>;

    /// Queries whether an exclusive purge operation is waiting for access.
    ///
    /// Background compactions inspect this periodically to cooperatively yield or abort.
    ///
    /// # Fehler
    /// Keine; nicht-blockierende atomare Abfrage.
    ///
    /// # Crash
    /// Reiner RAM-Zustandsprüfer.
    fn purge_waiting(&self) -> bool;
}

impl<T: CommitBarrier + ?Sized> CommitBarrier for Arc<T> {
    fn shared(&self) -> BoxFuture<'_, BarrierShared> {
        (**self).shared()
    }

    fn exclusive(&self) -> BoxFuture<'_, BarrierExclusive> {
        (**self).exclusive()
    }
}

impl<T: PurgeBarrier + ?Sized> PurgeBarrier for Arc<T> {
    fn compaction(&self) -> BoxFuture<'_, CompactionPermit> {
        (**self).compaction()
    }

    fn purge(&self) -> BoxFuture<'_, PurgePermit> {
        (**self).purge()
    }

    fn purge_waiting(&self) -> bool {
        (**self).purge_waiting()
    }
}
