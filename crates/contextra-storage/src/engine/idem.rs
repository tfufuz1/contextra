//! Idempotency token registry and cache (SPEC §11, SP-COMMIT, SD-67, INV-ATOM-1).
#![forbid(unsafe_code)]

use contextra_storage_ports::{Error, IdemToken, Stamp};
use std::collections::HashMap;

pub const DEFAULT_MAX_IDEM_ENTRIES: usize = 65_536;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdemState {
    Reserved,
    Committed { stamp: Stamp, durable: bool },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdemEntry {
    pub state: IdemState,
    pub request_digest: [u8; 32],
    pub expires_at_mono: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdemAction {
    New,
    Cached { stamp: Stamp, durable: bool },
}

pub struct IdemRegistry {
    entries: HashMap<IdemToken, IdemEntry>,
    max_entries: usize,
}

impl Default for IdemRegistry {
    fn default() -> Self {
        Self::new(DEFAULT_MAX_IDEM_ENTRIES)
    }
}

impl IdemRegistry {
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: HashMap::new(),
            max_entries: if max_entries == 0 {
                DEFAULT_MAX_IDEM_ENTRIES
            } else {
                max_entries
            },
        }
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Checks an incoming request against the idempotency cache.
    ///
    /// - If the token is already present with the same digest and committed, returns `Cached`.
    /// - If the token is present with a different digest, returns `Err(Error::IdemConflict)`.
    /// - If the token is present but still in `Reserved` state, returns `Err(Error::Conflict)`.
    /// - If the token is new, reserves it if capacity permits and returns `New`.
    pub fn check_or_reserve(
        &mut self,
        token: IdemToken,
        digest: [u8; 32],
        now_mono: u64,
        ttl_mono: u64,
    ) -> Result<IdemAction, Error> {
        if let Some(entry) = self.entries.get_mut(&token) {
            // Check if expired
            if now_mono >= entry.expires_at_mono {
                entry.state = IdemState::Reserved;
                entry.request_digest = digest;
                entry.expires_at_mono = now_mono.saturating_add(ttl_mono);
                return Ok(IdemAction::New);
            }

            if entry.request_digest != digest {
                return Err(Error::IdemConflict);
            }

            match entry.state {
                IdemState::Committed { stamp, durable } => {
                    return Ok(IdemAction::Cached { stamp, durable });
                }
                IdemState::Reserved => {
                    return Err(Error::Conflict);
                }
            }
        }

        if self.entries.len() >= self.max_entries {
            self.entries.retain(|_, e| {
                !matches!(e.state, IdemState::Reserved) || now_mono < e.expires_at_mono
            });
            if self.entries.len() >= self.max_entries {
                return Err(Error::IdemCapacity);
            }
        }

        self.entries.insert(
            token,
            IdemEntry {
                state: IdemState::Reserved,
                request_digest: digest,
                expires_at_mono: now_mono.saturating_add(ttl_mono),
            },
        );
        Ok(IdemAction::New)
    }

    /// Transitions a reserved token to committed status upon successful durable commit.
    pub fn commit(&mut self, token: IdemToken, stamp: Stamp, durable: bool) -> Result<(), Error> {
        if let Some(entry) = self.entries.get_mut(&token) {
            match entry.state {
                IdemState::Reserved => {
                    entry.state = IdemState::Committed { stamp, durable };
                    Ok(())
                }
                IdemState::Committed {
                    stamp: prev_stamp,
                    durable: prev_durable,
                } if prev_stamp == stamp && prev_durable == durable => Ok(()),
                IdemState::Committed { .. } => Err(Error::Conflict),
            }
        } else {
            Err(Error::Limit {
                limit: "unreserved idempotency token commit",
            })
        }
    }

    /// Aborts or removes a reserved token if the transaction failed before commit.
    pub fn abort(&mut self, token: IdemToken) {
        if let Some(entry) = self.entries.get(&token) {
            if entry.state == IdemState::Reserved {
                self.entries.remove(&token);
            }
        }
    }

    /// Trims idempotency records that are older than floor_stamp AND expired past now_mono.
    pub fn trim(&mut self, floor_stamp: Stamp, now_mono: u64) {
        self.entries.retain(|_, entry| match entry.state {
            IdemState::Reserved => now_mono < entry.expires_at_mono,
            IdemState::Committed { stamp, .. } => {
                // Keep if stamp is above floor OR unexpired
                stamp >= floor_stamp || now_mono < entry.expires_at_mono
            }
        });
    }
}
