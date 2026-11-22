//! # Synchronization Fassade
//!
//! Kapselt Sperren über `std::sync` (unter `cfg(loom)`: `loom::sync`).
//! Dies ist die einzige Stelle im gesamten Crate mit `cfg(loom)`.
//!
//! Unter `cfg(debug_assertions)` erzwingt `Ranked<T, RANK>` die Einhaltung
//! der Sperrhierarchie (INV-LOCK-1) zur Laufzeit.
//!
//! Vergiftete Sperren (Poisoning) werden übernommen (`PoisonError::into_inner`),
//! statt zu paniken. Die Signaturen liefern direkt den Guard; ein `Result` würde
//! alle Aufrufer ändern.

#![allow(clippy::collapsible_if)]

#[cfg(not(loom))]
pub use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
#[cfg(not(loom))]
pub use std::sync::{Arc, Condvar, Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};

#[cfg(loom)]
pub use loom::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
#[cfg(loom)]
pub use loom::sync::{Arc, Condvar, Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};

#[cfg(debug_assertions)]
use std::cell::RefCell;

#[cfg(debug_assertions)]
thread_local! {
    static LOCK_RANK_STACK: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
}

/// Prüft, ob ein Sperrrang erworben werden darf (INV-LOCK-1).
///
/// Verletzungen sind Programmierfehler der Sperrreihenfolge. Sie brechen nur im
/// Debug-Build ab. Ein `Err` ist hier nicht möglich, weil die Sperr-Signaturen
/// den Guard direkt liefern.
#[cfg(debug_assertions)]
fn push_rank(rank: u8) {
    LOCK_RANK_STACK.with(|stack| {
        let mut s = stack.borrow_mut();
        if let Some(&last) = s.last() {
            // INV-LOCK-1: Sperren nur in aufsteigender Rangfolge
            debug_assert!(
                last < rank,
                "INV-LOCK-1: Lock-Rang-Verletzung! Aktuell gehalten: {last}, angefordert: {rank}"
            );
        }
        s.push(rank);
    });
}

#[cfg(debug_assertions)]
fn pop_rank(rank: u8) {
    LOCK_RANK_STACK.with(|stack| {
        let mut s = stack.borrow_mut();
        if let Some(top) = s.pop() {
            debug_assert!(
                top == rank,
                "INV-LOCK-1: Lock-Rang-Stack inkonsistent! Erwartet: {rank}, gefunden: {top}"
            );
        }
    });
}

/// Rang-Token: hält einen Rang auf dem Thread-Stack, solange ein Guard lebt.
///
/// Trennt den Rang-Zustand vom Sperr-Guard, damit `wait` den Guard zerlegen kann,
/// ohne `unwrap` auf einen `Option` zu brauchen.
struct RankToken<const RANK: u8>;

impl<const RANK: u8> RankToken<RANK> {
    /// Meldet den Rang an (nur Debug-Build wirkt).
    fn acquire() -> Self {
        #[cfg(debug_assertions)]
        push_rank(RANK);
        Self
    }
}

impl<const RANK: u8> Drop for RankToken<RANK> {
    fn drop(&mut self) {
        #[cfg(debug_assertions)]
        pop_rank(RANK);
    }
}

/// Typisierte Sperre mit festem Rang (INV-LOCK-1).
pub struct RankedMutex<T, const RANK: u8> {
    inner: Mutex<T>,
}

impl<T, const RANK: u8> RankedMutex<T, RANK> {
    /// Erstellt einen neuen typisierten Mutex.
    pub fn new(val: T) -> Self {
        Self {
            inner: Mutex::new(val),
        }
    }

    /// Erwirbt die Sperre und prüft im Debug-Modus den Rang.
    pub fn lock(&self) -> RankedMutexGuard<'_, T, RANK> {
        let rank = RankToken::<RANK>::acquire();
        let guard = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        RankedMutexGuard { guard, _rank: rank }
    }

    /// Wartet atomar auf einer Condvar unter Beachtung des Rangstacks.
    pub fn wait<'a>(
        &self,
        cond: &Condvar,
        guard: RankedMutexGuard<'a, T, RANK>,
    ) -> RankedMutexGuard<'a, T, RANK> {
        let RankedMutexGuard { guard, _rank } = guard;
        // Rang abgeben, solange die Sperre während des Wartens frei ist
        drop(_rank);

        let guard = cond.wait(guard).unwrap_or_else(|e| e.into_inner());

        RankedMutexGuard {
            guard,
            _rank: RankToken::acquire(),
        }
    }

    /// Wartet mit Zeitüberschreitung atomar auf einer Condvar.
    #[cfg(not(loom))]
    pub fn wait_timeout<'a>(
        &self,
        cond: &Condvar,
        guard: RankedMutexGuard<'a, T, RANK>,
        dur: std::time::Duration,
    ) -> (RankedMutexGuard<'a, T, RANK>, std::sync::WaitTimeoutResult) {
        let RankedMutexGuard { guard, _rank } = guard;
        // Rang abgeben, solange die Sperre während des Wartens frei ist
        drop(_rank);

        let (guard, res) = cond
            .wait_timeout(guard, dur)
            .unwrap_or_else(|e| e.into_inner());

        (
            RankedMutexGuard {
                guard,
                _rank: RankToken::acquire(),
            },
            res,
        )
    }
}

/// Guard für einen `RankedMutex`.
pub struct RankedMutexGuard<'a, T, const RANK: u8> {
    guard: MutexGuard<'a, T>,
    // Feld wird nach `guard` freigegeben: erst entsperren, dann Rang abgeben
    _rank: RankToken<RANK>,
}

impl<T, const RANK: u8> std::ops::Deref for RankedMutexGuard<'_, T, RANK> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.guard
    }
}

impl<T, const RANK: u8> std::ops::DerefMut for RankedMutexGuard<'_, T, RANK> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.guard
    }
}

/// Typisierte RwLock mit festem Rang (INV-LOCK-1).
pub struct RankedRwLock<T, const RANK: u8> {
    inner: RwLock<T>,
}

impl<T, const RANK: u8> RankedRwLock<T, RANK> {
    /// Erstellt eine neue typisierte RwLock.
    pub fn new(val: T) -> Self {
        Self {
            inner: RwLock::new(val),
        }
    }

    /// Erwirbt die Lesesperre.
    pub fn read(&self) -> RankedRwLockReadGuard<'_, T, RANK> {
        let rank = RankToken::<RANK>::acquire();
        let guard = self.inner.read().unwrap_or_else(|e| e.into_inner());
        RankedRwLockReadGuard { guard, _rank: rank }
    }

    /// Erwirbt die Schreibsperre.
    pub fn write(&self) -> RankedRwLockWriteGuard<'_, T, RANK> {
        let rank = RankToken::<RANK>::acquire();
        let guard = self.inner.write().unwrap_or_else(|e| e.into_inner());
        RankedRwLockWriteGuard { guard, _rank: rank }
    }
}

/// Leseguard für eine `RankedRwLock`.
pub struct RankedRwLockReadGuard<'a, T, const RANK: u8> {
    guard: RwLockReadGuard<'a, T>,
    _rank: RankToken<RANK>,
}

impl<T, const RANK: u8> std::ops::Deref for RankedRwLockReadGuard<'_, T, RANK> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.guard
    }
}

/// Schreibguard für eine `RankedRwLock`.
pub struct RankedRwLockWriteGuard<'a, T, const RANK: u8> {
    guard: RwLockWriteGuard<'a, T>,
    _rank: RankToken<RANK>,
}

impl<T, const RANK: u8> std::ops::Deref for RankedRwLockWriteGuard<'_, T, RANK> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.guard
    }
}

impl<T, const RANK: u8> std::ops::DerefMut for RankedRwLockWriteGuard<'_, T, RANK> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.guard
    }
}
