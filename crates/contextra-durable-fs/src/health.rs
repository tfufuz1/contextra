// FILE-CONTEXT
// STAND: 2026-10-12
// ZWECK: Fail-Closed HealthState Zustandsautomat fuer Durability Substrat (§18.4).
// INVARIANTEN: Healthy Startzustand, Poisoned verlaesst Zustand nie automatisch ohne Neustart (I-POISON-1).

use std::sync::atomic::{AtomicU8, Ordering};

/// Health status classification of the storage subsystem (§18.4).
///
/// ## Transition Table & Rules
///
/// | Initial State | Trigger / Event | Target State | Permanent? | Action Required |
/// |--------------|-----------------|--------------|------------|-----------------|
/// | `Healthy`    | Read Error / Checksum Warning | `Degraded` | No | Scrubbing / Retry |
/// | `Healthy`    | Disk Full (`ENOSPC`) | `ReadOnly` | No | Cleanup / Space Release |
/// | Any          | `fsync` / Flush Write Failure | `Poisoned` | Yes | Full Restart + Recovery |
/// | `Degraded`   | `fsync` / Flush Write Failure | `Poisoned` | Yes | Full Restart + Recovery |
/// | `ReadOnly`   | `fsync` / Flush Write Failure | `Poisoned` | Yes | Full Restart + Recovery |
/// | `Poisoned`   | Any Event | `Poisoned` | Yes | Fail-closed write rejection |
///
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(u8)]
pub enum HealthState {
    /// Normal operational health with full read/write capabilities.
    #[default]
    Healthy = 0,
    /// Non-fatal read errors or checksum warnings encountered; write allowed, reads degraded.
    Degraded = 1,
    /// Storage target out of disk space (`ENOSPC`); write/commit rejected, read allowed.
    ReadOnly = 2,
    /// Fatal storage flush error (`fsync` failure); all writes rejected. Requires full restart.
    Poisoned = 3,
}

impl HealthState {
    /// Creates a new default [`HealthState`] initialized to `Healthy`.
    #[must_use]
    pub const fn new() -> Self {
        Self::Healthy
    }

    /// Evaluates state transition based on incoming system event or error.
    ///
    /// Once in `Poisoned`, transitions to any non-Poisoned state are strictly forbidden.
    #[must_use]
    pub fn transition(self, event: HealthEvent) -> Self {
        if self == Self::Poisoned {
            return Self::Poisoned;
        }

        match event {
            HealthEvent::FsyncFailure => Self::Poisoned,
            HealthEvent::Enospc => Self::ReadOnly,
            HealthEvent::ReadChecksumWarning => {
                if self == Self::ReadOnly {
                    Self::ReadOnly
                } else {
                    Self::Degraded
                }
            }
            HealthEvent::Recovered => Self::Healthy,
        }
    }

    /// Returns `true` if state allows mutation/write operations.
    #[must_use]
    pub fn allows_writes(self) -> bool {
        matches!(self, Self::Healthy | Self::Degraded)
    }

    /// Returns `true` if handle is permanently poisoned.
    #[must_use]
    pub fn is_poisoned(self) -> bool {
        matches!(self, Self::Poisoned)
    }
}

/// Triggers and health events driving state transitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HealthEvent {
    /// Failure during `fsync` or synchronous disk flush.
    FsyncFailure,
    /// Storage device out of space (`ENOSPC`).
    Enospc,
    /// Non-fatal read error or checksum verification warning.
    ReadChecksumWarning,
    /// Manual explicit recovery signal (e.g. upon process restart recovery).
    Recovered,
}

/// Atomic thread-safe wrapper holding current [`HealthState`].
#[derive(Debug)]
pub struct AtomicHealthState {
    state: AtomicU8,
}

impl Default for AtomicHealthState {
    fn default() -> Self {
        Self::new()
    }
}

impl AtomicHealthState {
    /// Creates a new atomic state initialized to `Healthy`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: AtomicU8::new(HealthState::Healthy as u8),
        }
    }

    /// Retrieves current [`HealthState`].
    #[must_use]
    pub fn load(&self) -> HealthState {
        match self.state.load(Ordering::Acquire) {
            0 => HealthState::Healthy,
            1 => HealthState::Degraded,
            2 => HealthState::ReadOnly,
            _ => HealthState::Poisoned,
        }
    }

    /// Triggers a state transition atomically.
    pub fn record_event(&self, event: HealthEvent) -> HealthState {
        let mut current = self.load();
        loop {
            let next = current.transition(event);
            match self.state.compare_exchange_weak(
                current as u8,
                next as u8,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => return next,
                Err(actual) => {
                    current = match actual {
                        0 => HealthState::Healthy,
                        1 => HealthState::Degraded,
                        2 => HealthState::ReadOnly,
                        _ => HealthState::Poisoned,
                    };
                }
            }
        }
    }
}
