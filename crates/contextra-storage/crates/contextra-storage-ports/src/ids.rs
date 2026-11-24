//! Identitätstypen. Debug ist redigiert, wo Kennungen Rückschlüsse erlauben (SP-PORTS §3.3).
use core::fmt;

pub const SYSTEM_SPACE_ID_MAX: u64 = 255;
pub const FIRST_USER_SPACE_ID: u64 = SYSTEM_SPACE_ID_MAX + 1;

/// Stamp = globale Versionsmarke, 56 Bit nutzbar.
pub type Stamp = u64;
pub const STAMP_MAX: Stamp = (1u64 << 56) - 1;

/// Umhüllt einen Wert, dessen Inhalt nie in Debug-Ausgaben erscheinen darf.
pub struct Redacted<T>(pub T);

impl<T> fmt::Debug for Redacted<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("<redacted>")
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ClusterId([u8; 16]);

impl ClusterId {
    /// Nur für den Wire-Decoder (SP-CODEC). Weitere Konstruktionspfade existieren nicht.
    #[doc(hidden)]
    pub fn from_wire(raw: [u8; 16]) -> Self {
        Self(raw)
    }
    pub fn as_bytes(&self) -> &[u8; 16] {
        &self.0
    }
}

impl fmt::Debug for ClusterId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ClusterId(<redacted>)")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SpaceId(pub u64);

impl SpaceId {
    pub const fn is_system(self) -> bool { self.0 <= SYSTEM_SPACE_ID_MAX }

    pub fn user(value: u64) -> Result<Self, crate::Error> {
        if value < FIRST_USER_SPACE_ID { return Err(crate::Error::ReservedSpace); }
        Ok(Self(value))
    }
}

/// Explicit classification at the storage API boundary (SPEC §4.7).
///
/// This is deliberately a closed enum.  Storage validates the combinations
/// involving this value before it starts catalog, key, or filesystem I/O; it
/// does not inspect user content to infer a classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataClass {
    NonPersonal,
    Personal,
}

/// Stable identity of one storage instance.  A stamp is meaningful only with
/// this identity and must never be treated as a globally ordered value.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InstanceId([u8; 16]);

impl InstanceId {
    /// Used by format and recovery codecs after validating the exact width.
    #[doc(hidden)]
    pub fn from_wire(raw: [u8; 16]) -> Self { Self(raw) }

    pub fn as_bytes(&self) -> &[u8; 16] { &self.0 }
}

impl fmt::Debug for InstanceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("InstanceId(<redacted>)")
    }
}

/// A durable progress point in one instance's stamp domain.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DurablePosition {
    pub instance: InstanceId,
    pub stamp: Stamp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BranchId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IdemToken([u8; 16]);

impl IdemToken {
    pub fn from_bytes(raw: [u8; 16]) -> Self {
        Self(raw)
    }
}

/// Separate idempotency domain for erase requests.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct EraseRequestId([u8; 16]);

impl EraseRequestId {
    pub fn from_bytes(raw: [u8; 16]) -> Self { Self(raw) }
}

impl fmt::Debug for EraseRequestId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("EraseRequestId(<redacted>)")
    }
}

/// Opaque durable erase-ticket identity.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Ticket([u8; 16]);

impl Ticket {
    pub fn from_bytes(raw: [u8; 16]) -> Self { Self(raw) }
    /// Canonical wire representation for durable internal records.
    pub fn as_bytes(&self) -> &[u8; 16] { &self.0 }
}

impl fmt::Debug for Ticket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Ticket(<redacted>)")
    }
}

/// Pseudonymous, domain-separated ledger identifier; never a clear ClusterId.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct TombId([u8; 32]);

impl TombId {
    pub fn from_bytes(raw: [u8; 32]) -> Self { Self(raw) }
    /// Canonical wire representation for the ledger only.
    pub fn as_bytes(&self) -> &[u8; 32] { &self.0 }
}

impl fmt::Debug for TombId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("TombId(<redacted>)")
    }
}
