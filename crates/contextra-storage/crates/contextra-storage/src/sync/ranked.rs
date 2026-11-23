//! Ranggeordnete Sperren. Ein Thread darf nur strikt aufsteigende Ränge halten.

use contextra_storage_ports::Error;
use std::cell::RefCell;
use std::ops::{Deref, DerefMut};
use std::sync::{Mutex, MutexGuard};

thread_local! {
    static HELD_RANKS: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
}

pub struct Ranked<T, const RANK: u8> {
    inner: Mutex<T>,
}

impl<T, const RANK: u8> Ranked<T, RANK> {
    pub const fn new(value: T) -> Self {
        Self { inner: Mutex::new(value) }
    }

    pub fn lock(&self) -> Result<RankedGuard<'_, T, RANK>, Error> {
        if RANK == 0 || RANK > 8 || !rank_is_valid(RANK) {
            return Err(Error::Limit { limit: "lock rank must increase within 1..=8" });
        }
        match self.inner.lock() {
            Ok(guard) => {
                push_rank(RANK);
                Ok(RankedGuard { guard, rank: RANK })
            }
            Err(_) => Err(Error::Poisoned { cause: "ranked mutex poisoned" }),
        }
    }

    pub fn into_inner(self) -> Result<T, Error> {
        self.inner.into_inner().map_err(|_| Error::Poisoned { cause: "ranked mutex poisoned" })
    }
}

pub struct RankedGuard<'a, T, const RANK: u8> {
    guard: MutexGuard<'a, T>,
    rank: u8,
}

impl<T, const RANK: u8> Deref for RankedGuard<'_, T, RANK> {
    type Target = T;
    fn deref(&self) -> &Self::Target { &self.guard }
}

impl<T, const RANK: u8> DerefMut for RankedGuard<'_, T, RANK> {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.guard }
}

impl<T, const RANK: u8> Drop for RankedGuard<'_, T, RANK> {
    fn drop(&mut self) {
        pop_rank(self.rank);
    }
}

/// Diagnose the held-rank stack without exposing thread-local state.
pub struct RankStack;

impl RankStack {
    pub fn current() -> Vec<u8> {
        HELD_RANKS.with(|ranks| ranks.borrow().clone())
    }
}

fn rank_is_valid(rank: u8) -> bool {
    HELD_RANKS.with(|ranks| ranks.borrow().last().is_none_or(|held| rank > *held))
}

fn push_rank(rank: u8) {
    HELD_RANKS.with(|ranks| ranks.borrow_mut().push(rank));
}

fn pop_rank(rank: u8) {
    HELD_RANKS.with(|ranks| {
        let mut ranks = ranks.borrow_mut();
        if let Some(index) = ranks.iter().rposition(|held| *held == rank) {
            ranks.remove(index);
        }
    });
}
