//! Atomare Generation `state:8 | gen:56` (SD-17, INV-ERA-2).

use std::sync::atomic::{AtomicU64, Ordering};

const GEN_MAX: u64 = (1u64 << 56) - 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EpochValue {
    pub state: u8,
    pub generation: u64,
}

pub struct Epoch(AtomicU64);

impl Epoch {
    pub const fn new(state: u8, generation: u64) -> Option<Self> {
        if generation <= GEN_MAX {
            Some(Self(AtomicU64::new(pack(state, generation))))
        } else {
            None
        }
    }

    pub fn load(&self) -> EpochValue {
        unpack(self.0.load(Ordering::Acquire))
    }

    /// Ändert den Zustand nur dann, wenn Zustand und Generation noch passen.
    /// Generation wird strikt erhöht und kann nicht umbrechen.
    pub fn transition(&self, expected: EpochValue, next_state: u8) -> Result<EpochValue, EpochValue> {
        let Some(next_generation) = expected.generation.checked_add(1).filter(|v| *v <= GEN_MAX) else {
            return Err(self.load());
        };
        let next = pack(next_state, next_generation);
        self.0.compare_exchange(
            pack(expected.state, expected.generation),
            next,
            Ordering::AcqRel,
            Ordering::Acquire,
        ).map(|_| EpochValue { state: next_state, generation: next_generation })
            .map_err(unpack)
    }
}

const fn pack(state: u8, generation: u64) -> u64 {
    ((state as u64) << 56) | generation
}

const fn unpack(value: u64) -> EpochValue {
    EpochValue { state: (value >> 56) as u8, generation: value & GEN_MAX }
}
