//! Monotone projection of cluster restrictions (SPEC §9.6).
//!
//! Mutations are deliberately separated from persistence. The storage
//! sequencer must publish `set`/`lift` only after their control frame is
//! durable, and recovery reconstructs this table by replaying those frames.

use contextra_storage_ports::{Error, Stamp};
use contextra_storage_ports::traits::{Authority, FourEyesProof, Principal};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Effects(u8);

impl Effects {
    pub const BLOCK_ERASE: u8 = 0b001;
    pub const BLOCK_WRITE: u8 = 0b010;
    pub const BLOCK_READ: u8 = 0b100;
    const VALID: u8 = Self::BLOCK_ERASE | Self::BLOCK_WRITE | Self::BLOCK_READ;

    pub fn new(bits: u8) -> Result<Self, Error> {
        if bits & !Self::VALID != 0 || bits & Self::BLOCK_ERASE == 0 {
            return Err(Error::Limit { limit: "restriction must include BLOCK_ERASE and known effects" });
        }
        Ok(Self(bits))
    }

    pub fn blocks_erase(self) -> bool { self.0 & Self::BLOCK_ERASE != 0 }
    pub fn blocks_write(self) -> bool { self.0 & Self::BLOCK_WRITE != 0 }
    pub fn blocks_read(self) -> bool { self.0 & Self::BLOCK_READ != 0 }
    pub fn bits(self) -> u8 { self.0 }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LiftPolicy { OwnerOrAdmin, FourEyes }

pub struct RestrictionSpec {
    pub reason: u16,
    pub owner: Principal,
    pub effects: Effects,
    /// Absolute monotone-clock tick. Expiry removes effects, never triggers erase.
    pub expires_at: Option<u64>,
}

pub struct Restriction {
    id: u64,
    reason: u16,
    owner: Principal,
    effects: Effects,
    expires_at: Option<u64>,
    set_stamp: Stamp,
    lifted_stamp: Option<Stamp>,
}

impl Restriction {
    pub fn id(&self) -> u64 { self.id }
    pub fn reason(&self) -> u16 { self.reason }
    pub fn effects(&self) -> Effects { self.effects }
    pub fn set_stamp(&self) -> Stamp { self.set_stamp }
    pub fn lifted_stamp(&self) -> Option<Stamp> { self.lifted_stamp }
    pub fn is_active_at(&self, monotone_now: u64) -> bool {
        self.lifted_stamp.is_none() && self.expires_at.is_none_or(|deadline| monotone_now < deadline)
    }
}

pub enum LiftAuthorization<'a> {
    Owner(&'a Principal),
    Admin(&'a AdminCap),
    FourEyes {
        requester: &'a Principal,
        approver: &'a Principal,
        proof: &'a FourEyesProof,
    },
}

/// Non-cloneable capability minted by the embedding authorization layer.
pub struct AdminCap { _seal: () }

impl AdminCap {
    /// Construct only after the caller has authenticated its administrative
    /// principal. The storage core does not implement identity policy.
    pub fn from_authorized_context() -> Self { Self { _seal: () } }
}

pub struct RestrictionTable {
    entries: BTreeMap<u64, Restriction>,
    next_id: u64,
    policy: LiftPolicy,
    last_stamp: Stamp,
}

impl RestrictionTable {
    pub fn new(policy: LiftPolicy) -> Self {
        Self { entries: BTreeMap::new(), next_id: 1, policy, last_stamp: 0 }
    }

    pub fn get(&self, id: u64) -> Option<&Restriction> { self.entries.get(&id) }
    pub fn iter(&self) -> impl Iterator<Item = &Restriction> { self.entries.values() }
    pub fn active_at(&self, now: u64) -> impl Iterator<Item = &Restriction> {
        self.entries.values().filter(move |restriction| restriction.is_active_at(now))
    }

    pub fn ensure_erase_allowed(&self, now: u64) -> Result<(), Error> {
        if self.active_at(now).any(|restriction| restriction.effects.blocks_erase()) {
            return Err(Error::Restricted);
        }
        Ok(())
    }

    pub fn ensure_write_allowed(&self, now: u64) -> Result<(), Error> {
        if self.active_at(now).any(|restriction| restriction.effects.blocks_write()) {
            return Err(Error::Restricted);
        }
        Ok(())
    }

    pub fn ensure_read_allowed(&self, now: u64) -> Result<(), Error> {
        if self.active_at(now).any(|restriction| restriction.effects.blocks_read()) {
            return Err(Error::Restricted);
        }
        Ok(())
    }

    /// Applies a previously validated, durable RestrictionSet control record.
    pub fn apply_set(&mut self, id: u64, stamp: Stamp, spec: RestrictionSpec) -> Result<(), Error> {
        if id == 0 || id != self.next_id || stamp <= self.last_stamp || spec.reason == 0 {
            return Err(Error::Corrupt { place: "restriction set sequence" });
        }
        if self.entries.contains_key(&id) { return Err(Error::Corrupt { place: "restriction id reused" }); }
        let next_id = id.checked_add(1).ok_or(Error::Exhausted)?;
        self.entries.insert(id, Restriction {
            id, reason: spec.reason, owner: spec.owner, effects: spec.effects,
            expires_at: spec.expires_at, set_stamp: stamp, lifted_stamp: None,
        });
        self.next_id = next_id;
        self.last_stamp = stamp;
        Ok(())
    }

    /// Applies a durable RestrictionLift. Authorization is checked before the
    /// caller writes the frame and must be revalidated by its commit sequencer.
    pub fn validate_lift(
        &self,
        id: u64,
        stamp: Stamp,
        authorization: LiftAuthorization<'_>,
        authority: Option<&dyn Authority>,
    ) -> Result<(), Error> {
        let restriction = self.entries.get(&id).ok_or(Error::Limit { limit: "unknown restriction" })?;
        if restriction.lifted_stamp.is_some() { return Ok(()); }
        if stamp <= self.last_stamp || stamp <= restriction.set_stamp { return Err(Error::Limit { limit: "restriction lift stamp must increase" }); }
        match (self.policy, authorization) {
            (LiftPolicy::OwnerOrAdmin, LiftAuthorization::Owner(owner)) if !owner.differs_from(&restriction.owner) => Ok(()),
            (LiftPolicy::OwnerOrAdmin, LiftAuthorization::Admin(_)) => Ok(()),
            (LiftPolicy::FourEyes, LiftAuthorization::FourEyes { requester, approver, proof }) => {
                if !approver.differs_from(requester) { return Err(Error::Restricted); }
                authority.ok_or(Error::Unsupported { what: "four-eyes authority provider" })?
                    .verify_four_eyes(requester, approver, proof)
            }
            _ => Err(Error::Restricted),
        }
    }

    pub fn apply_lift(&mut self, id: u64, stamp: Stamp) -> Result<(), Error> {
        let restriction = self.entries.get_mut(&id).ok_or(Error::Limit { limit: "unknown restriction" })?;
        if let Some(previous) = restriction.lifted_stamp {
            if previous == stamp { return Ok(()); }
            return Err(Error::Corrupt { place: "restriction lifted more than once" });
        }
        if stamp <= self.last_stamp || stamp <= restriction.set_stamp { return Err(Error::Corrupt { place: "restriction lift stamp order" }); }
        restriction.lifted_stamp = Some(stamp);
        self.last_stamp = stamp;
        Ok(())
    }
}
