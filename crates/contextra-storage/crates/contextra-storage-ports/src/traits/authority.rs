//! Optional authorization proof port for `LiftPolicy::FourEyes`.
//!
//! The core only verifies a supplied proof.  It never decides who is allowed
//! to lift a restriction, and it never persists principals or proof bytes.

use crate::error::Error;
use core::fmt;

#[derive(Clone, PartialEq, Eq)]
pub struct Principal(Vec<u8>);

impl Principal {
    pub fn from_opaque(bytes: Vec<u8>) -> Self { Self(bytes) }

    pub fn differs_from(&self, other: &Self) -> bool { self != other }
}

impl fmt::Debug for Principal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Principal(<redacted>)")
    }
}

pub struct FourEyesProof(Vec<u8>);

impl FourEyesProof {
    pub fn from_opaque(bytes: Vec<u8>) -> Self { Self(bytes) }

    pub fn as_bytes(&self) -> &[u8] { &self.0 }
}

impl fmt::Debug for FourEyesProof {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("FourEyesProof(<redacted>)")
    }
}

pub trait Authority: Send + Sync {
    /// Verifies the policy-layer proof for a lift.  The core rejects equal
    /// principals before calling this port; providers must not log the opaque
    /// values they receive.
    fn verify_four_eyes(
        &self,
        requester: &Principal,
        approver: &Principal,
        proof: &FourEyesProof,
    ) -> Result<(), Error>;
}
